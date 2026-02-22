//! <https://datatracker.ietf.org/doc/html/draft-sharabayko-srt#section-3.2.4>

use super::control_types;

#[derive(Clone, Debug)]
pub enum Ack {
    Full {
        // Header
        ack_number: u32,

        // CIF
        last_ackd_packet_sequence_number: u32,
        rtt: u32,
        rtt_variance: u32,
        available_buffer_size: u32,
        packets_receiving_rate: u32,
        estimated_link_capacity: u32,
        receiving_rate: u32,
    },
    Light {
        // CIF
        last_ackd_packet_sequence_number: u32,
    },
    Small {
        // CIF
        last_ackd_packet_sequence_number: u32,
        rtt: u32,
        rtt_variance: u32,
        available_buffer_size: u32,
    },
}

impl Ack {
    /// 44 BYTES
    pub fn from_raw(raw: &[u8]) -> anyhow::Result<Self> {
        match raw.len() {
            // Full
            44 => {
                let ack_number = u32::from_be_bytes(raw[4..8].try_into()?);

                let last_ackd_packet_sequence_number = u32::from_be_bytes(raw[16..20].try_into()?);
                let rtt = u32::from_be_bytes(raw[20..24].try_into()?);
                let rtt_variance = u32::from_be_bytes(raw[24..28].try_into()?);
                let available_buffer_size = u32::from_be_bytes(raw[28..32].try_into()?);
                let packets_receiving_rate = u32::from_be_bytes(raw[32..36].try_into()?);
                let estimated_link_capacity = u32::from_be_bytes(raw[36..40].try_into()?);
                let receiving_rate = u32::from_be_bytes(raw[40..44].try_into()?);

                Ok(Self::Full {
                    ack_number,
                    last_ackd_packet_sequence_number,
                    rtt,
                    rtt_variance,
                    available_buffer_size,
                    packets_receiving_rate,
                    estimated_link_capacity,
                    receiving_rate,
                })
            }
            // Light
            20 => {
                let last_ackd_packet_sequence_number = u32::from_be_bytes(raw[16..20].try_into()?);

                Ok(Self::Light {
                    last_ackd_packet_sequence_number,
                })
            }
            // Small
            32 => {
                let last_ackd_packet_sequence_number = u32::from_be_bytes(raw[16..20].try_into()?);
                let rtt = u32::from_be_bytes(raw[20..24].try_into()?);
                let rtt_variance = u32::from_be_bytes(raw[24..28].try_into()?);
                let available_buffer_size = u32::from_be_bytes(raw[28..32].try_into()?);

                Ok(Self::Small {
                    last_ackd_packet_sequence_number,
                    rtt,
                    rtt_variance,
                    available_buffer_size,
                })
            }
            _ => Err(anyhow::anyhow!("Wrong package size")),
        }
    }

    /// 8 BYTES
    pub fn raw_header(&self) -> Vec<u8> {
        let mut res = Vec::new();

        res.extend((control_types::ACK | (1 << 15)).to_be_bytes()); // Control Flag + Control Type
        res.extend(0u16.to_be_bytes()); // Reserved

        if let Self::Full { ack_number, .. } = self {
            res.extend(ack_number.to_be_bytes()); // Acknowledgement Number
        } else {
            res.extend([0; 4]);
        }

        res
    }

    /// BYTES (full)
    pub fn raw_content(&self) -> Vec<u8> {
        let mut res = Vec::new();

        match self {
            Ack::Full {
                last_ackd_packet_sequence_number,
                rtt,
                rtt_variance,
                available_buffer_size,
                packets_receiving_rate,
                estimated_link_capacity,
                receiving_rate,
                ..
            } => {
                res.extend(last_ackd_packet_sequence_number.to_be_bytes());
                res.extend(rtt.to_be_bytes());
                res.extend(rtt_variance.to_be_bytes());
                res.extend(available_buffer_size.to_be_bytes());
                res.extend(packets_receiving_rate.to_be_bytes());
                res.extend(estimated_link_capacity.to_be_bytes());
                res.extend(receiving_rate.to_be_bytes());
            }
            Ack::Light {
                last_ackd_packet_sequence_number,
            } => {
                res.extend(last_ackd_packet_sequence_number.to_be_bytes());
            }
            Ack::Small {
                last_ackd_packet_sequence_number,
                rtt,
                rtt_variance,
                available_buffer_size,
            } => {
                res.extend(last_ackd_packet_sequence_number.to_be_bytes());
                res.extend(rtt.to_be_bytes());
                res.extend(rtt_variance.to_be_bytes());
                res.extend(available_buffer_size.to_be_bytes());
            }
        }

        res
    }
}
