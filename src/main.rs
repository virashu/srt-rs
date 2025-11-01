use mpeg::packet::Packet as MpegPacket;
use srt::server::Server as SrtServer;
use tracing::Level;

fn run_hls() -> anyhow::Result<()> {
    tracing::info!("Starting HLS");
    hls::run();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

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

        match pack.packet_id {
            0x000 => println!("PAT"),
            0x100 => println!("Video"),
            0x101 => println!("Audio"),
            n => println!("0x{n:X}"),
        }
    });

    tracing::info!("Starting SRT");
    srt_server.run()?;

    Ok(())
}
