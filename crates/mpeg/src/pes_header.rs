#[derive(Debug)]
pub struct PesExtension {
    // Optional fields
    /// 128b?
    pub pes_private_data: Option<u128>,
    /// 8b?
    pub pack_header_field: Option<u8>,
    /// 8b?
    pub program_packet_seq_cntr: Option<u8>,
    /// 16b?
    pub pstd_buffer: Option<u16>,
    /// n?
    pub pes_extension_field_data: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum OriginalOrCopy {
    Original,
    Copy,
}

#[derive(Debug)]
pub struct PesHeader {
    // 2b const
    /// 2b
    pub pes_scrambling_control: u8,
    pub pes_priority: bool,
    /// 1b
    pub data_alignment_indicator: bool,
    /// 1b
    pub copyright: bool,
    /// 1b
    pub original_or_copy: OriginalOrCopy,

    // Optional fields
    /// 33b?
    pub pts_dts: Option<u64>,
    /// 42b?
    pub escr: Option<u64>,
    /// 22b?
    pub es_rate: Option<u32>,
    /// 8b?
    pub dsm_trick_mode: Option<u8>,
    /// 7b?
    pub additional_copy_info: Option<u8>,
    /// 16b?
    pub previous_pes_crc: Option<u16>,
    /// n?
    pub pes_extension: Option<PesExtension>,
}
