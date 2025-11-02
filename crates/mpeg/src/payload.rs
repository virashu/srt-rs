use crate::pes_packet::PesPacket;

/// n
#[derive(Debug)]
pub enum Payload {
    Pes(PesPacket),
    Psi,
    Data(Vec<u8>),
}

impl Payload {
    pub fn pes_from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        todo!()
    }
}
