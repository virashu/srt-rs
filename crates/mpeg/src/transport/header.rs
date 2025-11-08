use anyhow::{Result, anyhow};
use bit::Bit;

/// 4B
#[derive(Debug)]
pub struct Header {
    // 8b sync byte
    /// 1b
    pub transport_error_indicator: bool,
    /// 1b
    pub payload_unit_start_indicator: bool,
    /// 1b
    pub transport_priority: bool,
    /// 13b
    pub packet_id: u16,
    /// 2b
    pub transport_scrambling_control: u8,
    /// 2b
    pub adaptation_field_control: u8,
    /// 4b
    pub continuity_counter: u8,
}

impl Header {
    /// # Errors
    /// Error while parsing raw bytes
    pub fn from_raw(raw: &[u8]) -> Result<Self> {
        if raw[0] != 0x47 {
            return Err(anyhow!("Missing sync byte: {raw:?}"));
        }

        let transport_error_indicator = raw[1].bit(0);
        let payload_unit_start_indicator = raw[1].bit(1);
        let transport_priority = raw[1].bit(2);
        let packet_id = u16::from_be_bytes(raw[1..3].try_into()?) & 0b0001_1111_1111_1111;
        let transport_scrambling_control = (raw[3] & 0b1100_0000) >> 6;
        let adaptation_field_control = (raw[3] & 0b0011_0000) >> 4;
        let continuity_counter = raw[3] & 0b0000_1111;

        Ok(Self {
            transport_error_indicator,
            payload_unit_start_indicator,
            transport_priority,
            packet_id,
            transport_scrambling_control,
            adaptation_field_control,
            continuity_counter,
        })
    }

    pub const fn size(&self) -> usize {
        32
    }
}
