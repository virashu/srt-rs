use anyhow::Result;

use crate::{
    constants::packet_ids::GROUP_CONTROL,
    pes::packet::PesPacket,
    psi::packet::ProgramSpecificInformation,
    transport::{adaptation_field::AdaptationField, header::Header},
};

#[derive(Debug)]
pub enum AdaptationFieldOption {
    None,
    Empty,
    Some(AdaptationField),
}

#[derive(Debug)]
pub enum Payload {
    PES(PesPacket),
    PSI(ProgramSpecificInformation),
    Data(Vec<u8>),
}

#[derive(Debug)]
pub struct TransportPacket {
    pub header: Header,
    /// 16b+
    pub adaptation_field: AdaptationFieldOption,
    /// n
    pub payload: Option<Payload>,
}

impl TransportPacket {
    /// # Errors
    /// Error while parsing raw bytes
    pub fn from_raw(raw: &[u8], pmt_packet_ids: &[u16]) -> Result<Self> {
        let header = Header::from_raw(raw)?;

        let (adaptation_field_size, adaptation_field) =
            if header.adaptation_field_control & 0b10 == 0 {
                (0, AdaptationFieldOption::None)
            } else if raw[4] == 0 {
                (1, AdaptationFieldOption::Empty)
            } else {
                let a_f = AdaptationField::from_raw(&raw[4..])?;
                (a_f.size(), AdaptationFieldOption::Some(a_f))
            };

        let payload = if header.adaptation_field_control & 0b01 != 0 {
            let payload_body = &raw[(4 + adaptation_field_size)..];
            if header.payload_unit_start_indicator {
                // Contains PES or PSI
                if GROUP_CONTROL.contains(&header.packet_id)
                    || pmt_packet_ids.contains(&header.packet_id)
                {
                    Some(Payload::PSI(ProgramSpecificInformation::from_raw(
                        payload_body,
                    )?))
                } else {
                    Some(Payload::PES(PesPacket::from_raw(payload_body)?))
                }
            } else {
                Some(Payload::Data(Vec::from(payload_body)))
            }
        } else {
            None
        };

        Ok(Self {
            header,
            adaptation_field,
            payload,
        })
    }
}
