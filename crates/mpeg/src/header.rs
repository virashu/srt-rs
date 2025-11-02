//
//   0                   1                   2                   3
//   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |   Sync Byte   |T|U|P|        Package ID       |TSC|AFC|  CC   |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//

use bit::from_bits;

pub enum Pid {
    PAT,
    CAT,
    TSDT,
    ICIT,
}

/// 0-77b
#[derive(Debug)]
pub struct AdaptationFieldExtension {
    // Optional fields
    /// 1b?
    pub ltw_valid_flag: bool,
    /// 15b?
    pub ltw_offset: u16,
    // 2b empty
    /// 22b?
    pub piecewise_rate: u32,
    /// 4b?
    pub splice_type: u8,
    /// 33b?
    pub dts_next_au: u64,
}

/// 16b+
#[derive(Debug)]
pub struct AdaptationField {
    /// 8b
    pub adaptation_field_length: u8,
    /// 1b
    pub discontinuity_indicator: bool,
    /// 1b
    pub random_access_indicator: bool,
    /// 1b
    pub elementary_stream_priority_indicator: bool,
    /// 42b?
    pub pcr: Option<u64>,
    /// 42b?
    pub opcr: Option<u64>,
    /// 8b?
    pub splice_countdown: Option<u8>,
    /// n?
    pub transport_private_data: Option<Vec<u8>>,
    /// n?
    pub adaptation_field_extension_length: Option<u8>,
}

impl AdaptationField {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let adaptation_field_length = raw[4];
        let discontinuity_indicator = raw[5] & (1 << 7) != 0;
        let random_access_indicator = raw[5] & (1 << 6) != 0;
        let elementary_stream_priority_indicator = raw[5] & (1 << 5) != 0;
        let flags = raw[5] & 0b0001_1111;

        let mut bit_offset: usize = 48;

        let pcr = if flags & (1 << 4) != 0 {
            let r = Some(from_bits::<u64>(raw, bit_offset, 42));
            bit_offset += 42;
            r
        } else {
            None
        };
        let opcr = if flags & (1 << 3) != 0 {
            let r = Some(from_bits::<u64>(raw, bit_offset, 42));
            bit_offset += 42;
            r
        } else {
            None
        };
        let splice_countdown = if flags & (1 << 2) != 0 {
            let r = Some(from_bits::<u8>(raw, bit_offset, 8));
            bit_offset += 8;
            r
        } else {
            None
        };
        let transport_private_data = if flags & (1 << 1) != 0 {
            let len = from_bits::<u8>(raw, bit_offset, 8);
            bit_offset += 8;
            tracing::warn!(
                "Not implemented yet: mpeg::header::AdaptationField::from_raw.transport_private_data"
            );
            bit_offset += usize::from(len);
            None
        } else {
            None
        };
        let adaptation_field_extension_length = if flags & 1 != 0 {
            tracing::warn!(
                "Not implemented yet: mpeg::header::AdaptationField::from_raw.adaptation_field_extension_length"
            );
            None
        } else {
            None
        };

        Ok(Self {
            adaptation_field_length,
            discontinuity_indicator,
            random_access_indicator,
            elementary_stream_priority_indicator,
            pcr,
            opcr,
            splice_countdown,
            transport_private_data,
            adaptation_field_extension_length,
        })
    }

    pub fn size(&self) -> usize {
        16
    }
}

/// 24b+
#[derive(Debug)]
pub struct Header {
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
    /// n
    pub adaptation_field: AdaptationField,
}

impl Header {
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
        let adaptation_field = AdaptationField::from_raw(raw)?;

        Ok(Self {
            transport_error_indicator,
            payload_unit_start_indicator,
            transport_priority,
            packet_id,
            transport_scrambling_control,
            adaptation_field_control,
            continuity_counter,
            adaptation_field,
        })
    }

    pub fn size(&self) -> usize {
        24 + self.adaptation_field.size()
    }
}
