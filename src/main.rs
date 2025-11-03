use std::sync::{Arc, Mutex};

use mpeg::{
    constants::packet_ids::PROGRAM_ASSOCIATION_TABLE,
    psi::packet::{ProgramSpecificInformation, Section},
    transport::packet::{Payload, TransportPacket as MpegPacket},
};
use srt::{connection::Connection, server::Server as SrtServer};

fn run_hls() -> anyhow::Result<()> {
    tracing::info!("Starting HLS");
    hls::run();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let mut srt_server = SrtServer::new()?;

    srt_server.on_connect(&|conn| {
        tracing::info!(
            "Client connected: {:?}",
            conn.stream_id.clone().unwrap_or_default()
        );
    });

    srt_server.on_disconnect(&|conn| {
        tracing::info!(
            "Client disconnected: {:?}",
            conn.stream_id.clone().unwrap_or_default()
        );
    });

    let known_pids = Arc::new(Mutex::new(Vec::new()));

    let on_data = move |conn: &Connection, mpeg_data: &[u8]| {
        let id = conn.stream_id.clone().unwrap_or_default();
        // tracing::info!("Packet from {id}");

        for chunk in mpeg_data.chunks_exact(188) {
            let pack = MpegPacket::from_raw(chunk, &known_pids.lock().unwrap()).unwrap();

            if let Some(Payload::PSI(ProgramSpecificInformation {
                section: Section::PAS(table),
                ..
            })) = &pack.payload
            {
                let mut lock = known_pids.lock().unwrap();

                for assoc in &table.programs {
                    if !lock.contains(&assoc.program_id) {
                        lock.push(assoc.program_id);
                    }
                }
            }

            // if pack.header.packet_id == PROGRAM_ASSOCIATION_TABLE {
            //     tracing::info!("{pack:#?}");
            // } else if pack.header.packet_id == 0x1000 {
            //     tracing::info!("{pack:#?}");
            // }

            // match pack.header.packet_id {
            //     // System
            //     0x0000 => tracing::info!("PAT"),
            //     0x1000 => tracing::info!("PMT"),
            //     // User
            //     0x0100 => tracing::info!("Video"),
            //     0x0101 => tracing::info!("Audio"),

            //     n => tracing::info!("0x{n:X}"),
            // }
        }
    };

    let on_data: &'static _ = Box::leak(Box::new(on_data));
    srt_server.on_data(on_data);

    tracing::info!("Starting SRT");
    srt_server.run()?;

    Ok(())
}
