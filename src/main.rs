use std::{
    cell::RefCell,
    fs,
    io::Write,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use mpeg::{
    psi::packet::{ProgramSpecificInformation, Section},
    transport::packet::{Payload, TransportPacket as MpegPacket},
};
use srt::{connection::Connection, server::Server as SrtServer};

const SECONDS_PER_SEGMENT: u64 = 2;

fn run_srt(current_segment: Arc<Mutex<u64>>, is_ended: Arc<Mutex<bool>>) -> anyhow::Result<()> {
    let timer = Rc::new(RefCell::new(0u64));
    let current_segment_data = Rc::new(RefCell::new(Vec::<u8>::new()));

    let mut srt_server = SrtServer::new()?;

    let on_disconnect: &'static _ = Box::leak(Box::new(move |_: &Connection| {
        *is_ended.lock().unwrap() = true;
    }));
    srt_server.on_disconnect(on_disconnect);

    let on_data = move |_: &Connection, mpeg_data: &[u8]| {
        for chunk in mpeg_data.chunks_exact(188) {
            let pack = MpegPacket::from_raw(chunk, &[]).unwrap();

            // If packet is PAS
            if matches!(
                pack.payload,
                Some(Payload::PSI(ProgramSpecificInformation {
                    section: Section::PAS(_),
                    ..
                }))
            ) {
                let new_segment = *timer.borrow() / SECONDS_PER_SEGMENT;
                let mut current_segment = current_segment.lock().unwrap();

                // Flush
                if *current_segment != new_segment {
                    tracing::info!("segment-write {current_segment}");
                    fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(format!("_local/segment_{current_segment}.mpg"))
                        .unwrap()
                        .write_all(&current_segment_data.borrow())
                        .unwrap();
                    current_segment_data.borrow_mut().clear();
                }

                *current_segment = new_segment;
            }

            // If packet is Video (OBS)
            if pack.header.packet_id == 0x100
                && let Some(Payload::PES(pes)) = pack.payload
            {
                let seconds = pes.pes_header.unwrap().pts_dts.unwrap().pts() / 90_000;
                timer.replace(seconds);
            }

            current_segment_data.borrow_mut().extend(chunk);
        }
    };

    let on_data: &'static _ = Box::leak(Box::new(on_data));
    srt_server.on_data(on_data);

    tracing::info!("Starting SRT");
    srt_server.run()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    _ = fs::remove_dir_all("_local");
    fs::create_dir_all("_local").unwrap();

    // State
    let current_segment = Arc::new(Mutex::new(0u64));
    let is_ended = Arc::new(Mutex::new(false));

    thread::spawn({
        let current_segment = current_segment.clone();
        let is_ended = is_ended.clone();

        || run_srt(current_segment, is_ended).unwrap()
    });

    hls::run(SECONDS_PER_SEGMENT, current_segment, is_ended);

    Ok(())
}
