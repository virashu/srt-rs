mod handshake;
pub use handshake::Handshake;

use crate::util::parse_be;

pub mod control_types {
    pub const HANDSHAKE: u16 = 0x0000;
    pub const KEEPALIVE: u16 = 0x0001;
    pub const ACK: u16 = 0x0002;
    pub const NAK: u16 = 0x0003;
    pub const CONGESTION_WARNING: u16 = 0x0004;
    pub const SHUTDOWN: u16 = 0x0005;
    pub const ACKACK: u16 = 0x0006;
    pub const DROPREQ: u16 = 0x0007;
    pub const PEERERROR: u16 = 0x0008;
    pub const OTHER: u16 = 0x7FFF;
}

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum ControlType {
    Handshake = 0x0000,
    KeepAlive,
    Ack,
    Nak,
    CongestionWarning,
    Shutdown,
    AckAck,
    DropReq,
    PeerError,
    Other(u16),
}

impl From<u16> for ControlType {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => Self::Handshake,
            0x0001 => Self::KeepAlive,
            0x0002 => Self::Ack,
            0x0003 => Self::Nak,
            0x0004 => Self::CongestionWarning,
            0x0005 => Self::Shutdown,
            0x0006 => Self::AckAck,
            0x0007 => Self::DropReq,
            0x0008 => Self::PeerError,
            other => Self::Other(other),
        }
    }
}

pub enum ControlInformation {
    Handshake(Handshake),
}

impl ControlInformation {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let control_type = u16::from_be_bytes(raw[0..2].try_into()?) & !(1 << 15);
        let subtype = u16::from_be_bytes(raw[2..4].try_into()?);

        // Data after package header
        let content = &raw[16..];

        Ok(match control_type {
            control_types::HANDSHAKE => Self::Handshake(Handshake::from_raw(content)?),
            _ => todo!(),
        })
    }
}

#[derive(Debug)]
pub struct ControlPacket {
    pub control_type: ControlType,
    pub subtype: u16,
}

impl ControlPacket {
    pub fn from_raw(raw: &[u8]) -> Self {
        let control_type = (parse_be::<u16>(&raw[0..2]) & !(1 << 15)).into();
        let subtype = parse_be::<u16>(&raw[2..4]);

        Self {
            control_type,
            subtype,
        }
    }

    pub fn to_raw(&self) -> Vec<u8> {
        // let mut res = Vec::new();

        // res.extend((self.control_type as u16).to_be_bytes());
        // res.extend(self.subtype.to_be_bytes());

        // res
        
        todo!()
    }
}
