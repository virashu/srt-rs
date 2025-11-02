use crate::{header::Header, payload::Payload};

#[derive(Debug)]
pub struct Packet {
    pub header: Header,
    pub payload: Payload,
}

impl Packet {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let header = Header::from_raw(raw)?;

        let payload_body = &raw[header.size()..];

        let payload = if header.payload_unit_start_indicator {
            // Contains PES or PSI
            Payload::pes_from_raw(payload_body)?
        } else {
            Payload::Data(Vec::from(payload_body))
        };

        Ok(Self { header, payload })
    }
}
