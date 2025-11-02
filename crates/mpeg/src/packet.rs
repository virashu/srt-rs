use crate::{header::Header, payload::Payload};

#[derive(Debug)]
pub struct Packet {
    pub header: Header,
    pub payload: Payload,
}

impl Packet {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let header = Header::from_raw(raw)?;

        if header.payload_unit_start_indicator {
            // Contains PES or PSI
        }

        let payload = todo!();
    }
}
