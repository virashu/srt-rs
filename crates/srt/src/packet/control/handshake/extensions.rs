pub mod handshake_extension_flags {
    pub const HSREQ: u16 = 0x00_01;
    pub const KMREQ: u16 = 0x00_02;
    pub const CONFIG: u16 = 0x00_04;
}

pub mod handshake_extension_message_flags {
    pub const TSBPDSND: u32 = 0x00_00_00_01;
    pub const TSBPDRCV: u32 = 0x00_00_00_02;
    pub const CRYPT: u32 = 0x00_00_00_04;
    pub const TLPKTDROP: u32 = 0x00_00_00_08;
    pub const PERIODICNAK: u32 = 0x00_00_00_10;
    pub const REXMITFLG: u32 = 0x00_00_00_20;
    pub const STREAM: u32 = 0x00_00_00_40;
    pub const PACKET_FILTER: u32 = 0x00_00_00_80;
}

#[derive(Clone, Debug)]
pub struct HandshakeExtension {
    pub r#type: u16,
    pub length: u16,
    pub srt_version: u32,
    pub srt_flags: u32,
    pub receiver_delay: u16,
    pub sender_delay: u16,
}

impl HandshakeExtension {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let r#type = u16::from_be_bytes(raw[0..2].try_into()?);
        let length = u16::from_be_bytes(raw[2..4].try_into()?);

        let srt_version = u32::from_be_bytes(raw[4..8].try_into()?);
        let srt_flags = u32::from_be_bytes(raw[8..12].try_into()?);
        let receiver_delay = u16::from_be_bytes(raw[12..14].try_into()?);
        let sender_delay = u16::from_be_bytes(raw[14..16].try_into()?);

        Ok(Self {
            r#type,
            length,
            srt_version,
            srt_flags,
            receiver_delay,
            sender_delay,
        })
    }

    pub fn to_raw(&self) -> Vec<u8> {
        let mut res = Vec::new();

        res.extend(self.r#type.to_be_bytes());
        res.extend(self.length.to_be_bytes());
        res.extend(self.srt_version.to_be_bytes());
        res.extend(self.srt_flags.to_be_bytes());
        res.extend(self.receiver_delay.to_be_bytes());
        res.extend(self.sender_delay.to_be_bytes());

        res
    }
}

#[derive(Clone, Debug)]
pub enum KeyBasedEncryption {
    // None,
    EvenKey,
    OddKey,
    Both,
}

#[derive(Clone, Debug)]
pub struct KeyMaterialExtension {
    pub r#type: u16,
    pub length: u16,
    pub packet_type: u8,
    pub key_based_encryption: KeyBasedEncryption,
}

impl KeyMaterialExtension {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let r#type = u16::from_be_bytes(raw[0..2].try_into()?);
        let length = u16::from_be_bytes(raw[2..4].try_into()?);

        let packet_type = raw[4] & 0b0000_1111;
        // let sign = u16::from_be_bytes(raw[5..7].try_into()?); // = 0x2029
        let key_based_encryption = match raw[7] & 0x0000_0011 {
            0b00 => return Err(anyhow::anyhow!("Invalid extension format")),
            0b01 => KeyBasedEncryption::EvenKey,
            0b10 => KeyBasedEncryption::OddKey,
            0b11 => KeyBasedEncryption::Both,
            _ => unreachable!(),
        };
        // let keki = u32::from_be_bytes(raw[8..12].try_into()?); // = 0
        let cipher = u8::from_be_bytes(raw[12..13].try_into()?);
        // let auth = u8::from_be_bytes(raw[13..14].try_into()?); // = 0
        let stream_encapsulation = u8::from_be_bytes(raw[14..15].try_into()?);

        Ok(Self {
            r#type,
            length,
            packet_type,
            key_based_encryption,
        })
    }

    pub fn to_raw(&self) -> Vec<u8> {
        let mut res = Vec::new();

        res
    }
}

#[derive(Clone, Debug)]
pub struct StreamIdExtension {
    pub r#type: u16,
    pub length: u16,
    pub stream_id: String,
}

impl StreamIdExtension {
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        let r#type = u16::from_be_bytes(raw[0..2].try_into()?);
        let length = u16::from_be_bytes(raw[2..4].try_into()?);

        let mut stream_id = String::new();

        for i in 0..length {
            let pos = 4 + i as usize * 4;
            let mut bytes = Vec::from(&raw[pos..(pos + 4)]);
            bytes.reverse();
            stream_id += String::from_utf8_lossy(&bytes).trim_matches(char::from(0));
        }

        Ok(Self {
            r#type,
            length,
            stream_id,
        })
    }

    pub fn to_raw(&self) -> Vec<u8> {
        let mut res = Vec::new();

        res
    }
}
