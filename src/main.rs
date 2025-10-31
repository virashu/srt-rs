use srt::connection::connect;
use tracing::Level;

use std::{
    fs::{self, OpenOptions},
    io::Write,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let callback = |mpeg_packet: &[u8]| {
        let mut file = OpenOptions::new().append(true).open("a.mpg").unwrap();
        file.write_all(mpeg_packet).unwrap();
    };

    fs::write("a.mpg", []).unwrap();
    loop {
        connect(&callback)?;
    }
}
