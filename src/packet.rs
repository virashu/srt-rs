pub mod control;
pub mod data;

use crate::{
    packet::{control::ControlPacket, data::DataPacket},
    util::raw_u32_be,
};

#[derive(Debug)]
pub enum PacketContent {
    Data(DataPacket),
    Control(ControlPacket),
}

/// [ 16 BYTES ]
#[derive(Debug)]
pub struct Packet {
    pub timestamp: u32,
    pub dest_socket_id: u32,

    pub content: PacketContent,
}

impl Packet {
    pub fn from_raw(raw: &[u8]) -> Self {
        // Common properties
        let timestamp = raw_u32_be(&raw[8..12]);
        let dest_socket_id = raw_u32_be(&raw[12..16]);

        // Type-dependent properties
        let packet_type = (raw[0] & 0b1000_0000) >> 7;

        let content = match packet_type {
            1 => PacketContent::Control(ControlPacket::from_raw(raw)),
            0 => PacketContent::Data(DataPacket::from_raw(raw)),
            _ => unreachable!(),
        };

        Self {
            timestamp,
            dest_socket_id,
            content,
        }
    }

    pub fn to_raw(&self) -> Vec<u8> {
        let mut res = Vec::new();

        res.extend(self.timestamp.to_be_bytes());
        res.extend(self.dest_socket_id.to_be_bytes());
        res.extend(match &self.content {
            PacketContent::Control(p) => p.to_raw(),
            PacketContent::Data(p) => p.to_raw(),
        });

        res
    }
}
