use mpeg::{packet::Packet as MpegPacket, payload::Payload, pes_packet::PesPacket};
use srt::server::Server as SrtServer;

fn run_hls() -> anyhow::Result<()> {
    tracing::info!("Starting HLS");
    hls::run();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rs_srt=info,mpeg=off")
        .init();

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

    srt_server.on_data(&|conn, mpeg_packet| {
        let id = conn.stream_id.clone().unwrap_or_default();

        let pack = MpegPacket::from_raw(mpeg_packet).unwrap();

        match pack.header.packet_id {
            0x000 => tracing::info!("PAT"),
            0x100 => tracing::info!("Video"),
            0x101 => tracing::info!("Audio"),
            n => tracing::info!("0x{n:X}"),
        }

        if let Payload::Pes(PesPacket {
            pes_header: Some(header),
            ..
        }) = pack.payload
        {
            tracing::info!("{:#x?}", header);
        }
    });

    tracing::info!("Starting SRT");
    srt_server.run()?;

    Ok(())
}
