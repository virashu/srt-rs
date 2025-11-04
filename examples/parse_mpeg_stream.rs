use std::sync::{Arc, Mutex};

use mpeg::{
    psi::packet::{ProgramSpecificInformation, Section},
    transport::packet::{Payload, TransportPacket as MpegPacket},
};
use srt::{connection::Connection, server::Server as SrtServer};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let mut srt_server = SrtServer::new()?;

    srt_server.on_connect(|conn| {
        tracing::info!(
            "Client connected: {:?}",
            conn.stream_id.clone().unwrap_or_default()
        );
    });

    srt_server.on_disconnect(|conn| {
        tracing::info!(
            "Client disconnected: {:?}",
            conn.stream_id.clone().unwrap_or_default()
        );
    });

    let pids_pmt = Arc::new(Mutex::new(Vec::new()));

    let on_data = move |conn: &Connection, mpeg_data: &[u8]| {
        let id = conn.stream_id.clone().unwrap_or_default();
        tracing::info!("Packet from {id:?}");

        for chunk in mpeg_data.chunks_exact(188) {
            let pack = MpegPacket::from_raw(chunk, &pids_pmt.lock().unwrap()).unwrap();

            if let Some(Payload::PSI(ProgramSpecificInformation {
                section: Section::PAS(table),
                ..
            })) = &pack.payload
            {
                let mut lock = pids_pmt.lock().unwrap();

                for assoc in &table.programs {
                    if !lock.contains(&assoc.program_id) {
                        lock.push(assoc.program_id);
                    }
                }
            }

            match pack.header.packet_id {
                // System
                0x0000 => tracing::info!("PAT"),

                // User
                0x0100 => tracing::info!("OBS Video"),
                0x0101 => tracing::info!("OBS Audio 1"),
                0x0102 => tracing::info!("OBS Audio 2"),

                // Dynamically-assigned PMT
                n if pids_pmt.lock().unwrap().contains(&n) => tracing::info!("PMT"),

                n => tracing::info!("Other: 0x{n:X}"),
            }
        }
    };

    srt_server.on_data(on_data);

    tracing::info!("Starting SRT");
    srt_server.run()?;

    Ok(())
}
