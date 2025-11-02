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
    
}
