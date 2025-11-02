use crate::pes_header::PesHeader;

/// 24b+
#[derive(Debug)]
pub struct PesPacket {
    /// 8b
    pub stream_id: u8,
    /// 16b
    pub pes_packet_length: u16,
    /// n?
    pub pes_header: Option<PesHeader>,
    /// n
    pub pes_data: Vec<u8>,
}

impl PesPacket {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let stream_id = raw[3];
        let pes_packet_length = u16::from_be_bytes(raw[4..6].try_into()?);

        let pes_header = if raw[6] & 0b1100_0000 == 0b1000_0000 {
            Some(PesHeader::from_raw(&raw[6..])?)
        } else {
            None
        };

        let data_start = 6 + pes_header.as_ref().map_or(0, PesHeader::size);
        let pes_data = Vec::from(&raw[data_start..]);

        Ok(Self {
            stream_id,
            pes_packet_length,
            pes_header,
            pes_data,
        })
    }
}
