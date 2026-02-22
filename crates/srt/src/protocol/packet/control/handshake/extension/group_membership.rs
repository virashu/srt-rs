//! <https://datatracker.ietf.org/doc/html/draft-sharabayko-srt#section-3.2.1.4>

pub mod group_type {
    pub const UNDEFINED: u32 = 0;
    pub const BROADCAST: u32 = 1;
    pub const MAIN_BACKUP: u32 = 2;
    pub const BALANCING: u32 = 3;
    pub const MULTICAST: u32 = 4;
}

pub struct GroupMembershipExtension {
    pub group_id: u32,
    pub r#type: u8,
    pub flags: u8,
    pub weight: u16,
}

impl GroupMembershipExtension {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let group_id = u32::from_be_bytes(raw[4..8].try_into()?);
        let r#type = raw[8];
        let flags = raw[9];
        let weight = u16::from_be_bytes(raw[10..12].try_into()?);

        Ok(Self {
            group_id,
            r#type,
            flags,
            weight,
        })
    }
}
