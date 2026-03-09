use std::{
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::{Instant, SystemTime},
};

use anyhow::{Result, anyhow, bail};
use tokio::sync::Mutex;

use crate::{
    protocol::{
        constants::{FULL_ACK_INTERVAL, HANDSHAKE_MAGIC_CODE, RTT_INIT, RTT_VAR_INIT},
        packet::{
            Packet,
            PacketContent,
            control::{ControlPacketInfo, ack::Ack, handshake::Handshake, nak::Nak},
            data::DataPacketInfo,
        },
    },
    server::tokio::listener::Stream,
};

pub struct AsyncConnection {
    // Srt info
    pub stream_id: Option<String>,
    pub established: SystemTime,
    pub peer_srt_socket_id: u32,

    stream: Stream,
    running: AtomicBool,

    /// Ack sequence number
    ack_counter: AtomicU32,

    /// Timestamp of the last **sent** Ack
    /// (used to calculate RTT)
    last_ack_timestamp: Mutex<Instant>,

    /// Package sequence number of last received data packet
    last_received: AtomicU32,

    /// <add link>
    rtt: AtomicU32,
    /// <add link>
    rtt_var: AtomicU32,
}

impl AsyncConnection {
    pub async fn establish_v5(mut stream: Stream) -> Result<Self> {
        //
        // Induction phase
        //

        let in_packet_0 = stream.recv().await.unwrap();
        let PacketContent::Control(ControlPacketInfo::Handshake(handshake)) = in_packet_0.content
        else {
            bail!("Failed to unwrap handshake");
        };

        let out_packet_0_v5 = Packet {
            timestamp: in_packet_0.timestamp + 1,
            dest_socket_id: handshake.srt_socket_id,
            content: PacketContent::Control(ControlPacketInfo::Handshake(Handshake {
                version: 5,
                extension_field: HANDSHAKE_MAGIC_CODE,
                srt_socket_id: 42,
                syn_cookie: 42,
                ..handshake
            })),
        };
        stream.send(out_packet_0_v5).await?;

        tracing::debug!("Completed Induction");

        //
        // Conclusion phase
        //

        let in_packet_1 = stream.recv().await.unwrap();
        let PacketContent::Control(ControlPacketInfo::Handshake(handshake)) = in_packet_1.content
        else {
            bail!("Failed to unwrap handshake");
        };

        let peer_srt_socket_id = handshake.srt_socket_id;
        let stream_id = handshake
            .stream_id_extension
            .as_ref()
            .map(|x| x.stream_id.clone());

        let out_packet_1_v5 = Packet {
            timestamp: in_packet_1.timestamp + 1,
            dest_socket_id: handshake.srt_socket_id,
            content: PacketContent::Control(ControlPacketInfo::Handshake(handshake)),
        };
        stream.send(out_packet_1_v5).await?;

        tracing::debug!("Completed Conclusion");

        //
        // Done
        //

        tracing::debug!("Completed Handshake");

        let established = SystemTime::now();

        Ok(Self::new(stream, stream_id, established, peer_srt_socket_id).await)
    }

    async fn new(
        stream: Stream,
        stream_id: Option<String>,
        established: SystemTime,
        peer_srt_socket_id: u32,
    ) -> Self {
        Self {
            stream_id,
            established,
            peer_srt_socket_id,

            stream,
            running: AtomicBool::new(true),

            ack_counter: AtomicU32::new(1),
            last_ack_timestamp: Mutex::new(Instant::now()),
            last_received: AtomicU32::new(0),

            rtt: AtomicU32::new(RTT_INIT),
            rtt_var: AtomicU32::new(RTT_VAR_INIT),
        }
    }

    pub(crate) fn inc_ack(&self) -> u32 {
        self.ack_counter.fetch_add(1, Ordering::Relaxed)
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn pack(&self, content: PacketContent) -> Result<Packet> {
        Ok(Packet {
            timestamp: SystemTime::now()
                .duration_since(self.established)?
                .as_micros() as u32,
            dest_socket_id: self.peer_srt_socket_id,
            content,
        })
    }

    pub async fn send(&self, content: PacketContent) -> Result<()> {
        self.stream.send(self.pack(content)?).await?;

        Ok(())
    }

    async fn handle_control(&self, control: &ControlPacketInfo) -> Result<()> {
        tracing::trace!("srt | inbound | control | {control:?}");

        match control {
            ControlPacketInfo::KeepAlive => {
                let keep_alive = PacketContent::Control(ControlPacketInfo::KeepAlive);
                tracing::trace!("srt | outbound | control | {keep_alive:?}");
                self.send(keep_alive).await?;
            }
            ControlPacketInfo::AckAck(_) => {
                // Calculate RTT
                // RTT = 7/8 * RTT + 1/8 * rtt
                // RTTVar = 3/4 * RTTVar + 1/4 * abs(RTT - rtt)

                let sent = self.last_ack_timestamp.lock().await;
                let rtt_new: u32 = sent.elapsed().as_micros().try_into().unwrap();

                #[allow(
                    clippy::unwrap_used,
                    reason = "always `Ok()`; awaiting for `atomic_try_update` feature"
                )]
                let rtt_old = self
                    .rtt
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                        Some(x * 7 / 8 + rtt_new / 8)
                    })
                    .unwrap();

                #[allow(
                    clippy::unwrap_used,
                    reason = "always `Ok()`; awaiting for `atomic_try_update` feature"
                )]
                self.rtt_var
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |rtt_var| {
                        Some(rtt_var * 3 / 4 + (rtt_old).abs_diff(rtt_new) / 4)
                    })
                    .unwrap();
            }
            ControlPacketInfo::Shutdown => {
                self.running.store(false, Ordering::Relaxed);
            }
            _ => (),
        }

        Ok(())
    }

    async fn handle_data(&self, data_packet: &DataPacketInfo) -> Result<Box<[u8]>> {
        let packet_number = data_packet.packet_sequence_number;
        let prev_packet_number = self.last_received.swap(packet_number, Ordering::Relaxed);

        if prev_packet_number + 1 != packet_number && data_packet.message_number != 1 {
            tracing::warn!("Missed {} packets", packet_number - prev_packet_number - 1);

            self.send(PacketContent::Control(ControlPacketInfo::Nak(
                Nak::Single {
                    lost_packet: packet_number - 1,
                },
            )))
            .await?;
        }

        tracing::trace!(
            "srt | inbound | data | Data {{ packet_sequence_number: {:?}, position: {:?}, order: {:?}, encryption: {:?}, retransmitted: {:?}, message_number: {:?}, length: {:?} }}",
            data_packet.packet_sequence_number,
            data_packet.position,
            data_packet.order,
            data_packet.encryption,
            data_packet.retransmitted,
            data_packet.message_number,
            data_packet.content.len()
        );

        self.send_full_ack().await?;

        let data = &data_packet.content[..];

        Ok(Box::from(data))
    }

    async fn update(&self) -> Result<()> {
        let mut last_ack_timestamp = self.last_ack_timestamp.lock().await;
        let micros: u32 = last_ack_timestamp.elapsed().as_micros().try_into()?;

        if micros > FULL_ACK_INTERVAL {
            *last_ack_timestamp = Instant::now();
            self.send_full_ack().await?;
        }

        Ok(())
    }

    pub(crate) async fn handle(&self, pack: &Packet) -> Result<()> {
        self.update().await?;

        match &pack.content {
            PacketContent::Control(control) => self.handle_control(control).await?,
            PacketContent::Data(data) => {
                _ = self.handle_data(data).await?;
            }
        }

        Ok(())
    }

    pub async fn recv_data(&mut self) -> Result<Box<[u8]>> {
        if !self.running.load(Ordering::Relaxed) {
            return Err(anyhow!("Shut down"));
        }

        self.update().await?;

        let data = loop {
            let pack = self.stream.recv().await.ok_or(anyhow!("Recv error"))?;

            match &pack.content {
                PacketContent::Control(control) => self.handle_control(control).await?,
                PacketContent::Data(data) => {
                    let data = self.handle_data(data).await?;
                    break data;
                }
            }
        };

        Ok(data)
    }

    async fn send_full_ack(&self) -> Result<()> {
        let ack = PacketContent::Control(ControlPacketInfo::Ack(Ack::Full {
            ack_number: self.inc_ack(),
            last_ackd_packet_sequence_number: self.last_received.load(Ordering::Relaxed) + 1,
            rtt: self.rtt.load(Ordering::Relaxed),
            rtt_variance: self.rtt_var.load(Ordering::Relaxed),
            available_buffer_size: 1,
            packets_receiving_rate: 1,
            estimated_link_capacity: 1,
            receiving_rate: 1,
        }));
        tracing::trace!("srt | outbound | control | {ack:?}");
        self.send(ack).await
    }
}
