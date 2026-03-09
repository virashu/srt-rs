use std::sync::Arc;

use anyhow::Result;
use mpeg::{
    psi::packet::{ProgramSpecificInformation, Section},
    transport::packet::{Payload, TransportPacket as MpegPacket},
};
use srt::{AsyncConnection as SrtConnection, AsyncListener as SrtListener};
use tokio::sync::Mutex;

async fn handle_connection(mut con: SrtConnection) -> Result<()> {
    let stream_id = con.stream_id.clone().unwrap_or_default();
    tracing::info!("Client connected: {stream_id:?}",);

    let pids_pmt = Arc::new(Mutex::new(Vec::<u16>::new()));

    while let Ok(data) = con.recv_data().await {
        tracing::info!("Packet from {stream_id:?}");

        for chunk in data.chunks_exact(188) {
            let pack = MpegPacket::from_raw(chunk, &pids_pmt.lock().await).unwrap();

            match pack.payload {
                Some(Payload::PSI(ProgramSpecificInformation {
                    section: Section::PAS(table),
                    ..
                })) => {
                    tracing::info!("{table:#?}");
                    let mut lock = pids_pmt.lock().await;

                    for assoc in &table.programs {
                        if !lock.contains(&assoc.program_id) {
                            lock.push(assoc.program_id);
                        }
                    }
                }

                Some(Payload::PSI(ProgramSpecificInformation {
                    section: Section::PMS(table),
                    ..
                })) => {
                    tracing::info!("{table:#?}");
                }

                _ => {}
            }

            match pack.header.packet_id {
                // User
                0x0100 => tracing::info!("OBS Video"),
                0x0101 => tracing::info!("OBS Audio 1"),
                0x0102 => tracing::info!("OBS Audio 2"),

                n => tracing::info!("Other: 0x{n:X}"),
            }
        }
    }

    tracing::info!("Client disconnected: {stream_id:?}",);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    tracing::info!("Starting SRT");
    let listener = SrtListener::bind("0.0.0.0:1935").await?;

    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.poll_next().await {
        let connection = SrtConnection::establish_v5(stream).await?;

        handle_connection(connection).await?;
    }

    Ok(())
}
