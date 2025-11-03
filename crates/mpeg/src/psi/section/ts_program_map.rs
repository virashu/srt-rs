use bit::{Bit, Bits};

#[derive(Debug)]
pub struct TsProgramMapSection {
    pub table_id: u8,
    pub section_syntax_indicator: bool,
    pub section_length: u16,
    pub program_number: u16,
    pub version_number: u8,
    pub current_next_indicator: bool,
    pub section_number: u8,
    pub last_section_number: u8,

    pub pcr_pid: u16,
    pub program_info_length: u16,

    pub crc_32: u32,
}

impl TsProgramMapSection {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_MPEG_2);

        let table_id = raw[0];
        let section_syntax_indicator = raw[1].bit(0);
        let section_length = u16::from_be_bytes(raw[1..3].try_into()?) & !(0b11_11_11 << 10);
        let program_number = u16::from_be_bytes(raw[3..5].try_into()?);
        let version_number = raw[5..].bits::<u8>(2, 5);
        let current_next_indicator = raw[5..].bit(7);
        let section_number = raw[6];
        let last_section_number = raw[7];

        let pcr_pid = u16::from_be_bytes(raw[8..10].try_into()?) & !(0b111 << 13);
        let program_info_length = u16::from_be_bytes(raw[10..12].try_into()?) & !(0b1111 << 12);

        let crc_32 = raw[(section_length as usize - 1)..].bits::<u32>(0, 32);

        // Check CRC
        let checksum = CRC.checksum(&raw[0..(section_length as usize - 1)]);
        if checksum != crc_32 {
            return Err(anyhow::anyhow!(
                "Checksum does not match: {checksum} != {crc_32}"
            ));
        }

        Ok(Self {
            table_id,
            section_syntax_indicator,
            section_length,
            program_number,
            version_number,
            current_next_indicator,
            section_number,
            last_section_number,
            pcr_pid,
            program_info_length,
            crc_32,
        })
    }
}
