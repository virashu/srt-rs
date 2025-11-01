//   0                   1                   2                   3
//   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |   Sync Byte   |T|U|P|        Package ID       |TSC|AFC|  CC   |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//

pub enum Pid {
    PAT,
    CAT,
    TSDT,
    ICIT,
}

#[derive(Debug)]
pub struct Packet {
    pub transport_error_indicator: bool,
    pub payload_unit_start_indicator: bool,
    pub transport_priority: bool,
    pub packet_id: u16,
    pub transport_scrambling_control: u8,
    pub adaptation_field_control: u8,
    pub continuity_counter: u8,
}

impl Packet {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        if raw[0] != 0x47 {
            return Err(anyhow::anyhow!("Missing sync byte"));
        }

        let transport_error_indicator = raw[1] & (1 << 7) != 0;
        let payload_unit_start_indicator = raw[1] & (1 << 6) != 0;
        let transport_priority = raw[1] & (1 << 5) != 0;
        let packet_id = u16::from_be_bytes(raw[1..3].try_into()?) & 0b0001_1111_1111_1111;
        let transport_scrambling_control = raw[3] & 0b1100_0000 >> 6;
        let adaptation_field_control = raw[3] & 0b0011_0000 >> 4;
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
}
