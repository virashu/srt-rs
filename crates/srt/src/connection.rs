use std::{
    net::{SocketAddr, UdpSocket},
    sync::atomic::{AtomicU32, Ordering},
    time::SystemTime,
};

use crate::{
    ops::{handshake_v5, make_ack},
    packet::{Packet, PacketContent, control::ControlPacketInfo},
};

/// Maybe some config??
pub struct Connection {
    pub stream_id: Option<String>,
    pub est: SystemTime,
    pub addr: SocketAddr,
    pub peer_srt_socket_id: u32,
    pub ack_counter: AtomicU32,
}

impl Connection {
    pub fn listen_v5(socket: &UdpSocket) -> anyhow::Result<Self> {
        handshake_v5(socket)
    }

    pub fn inc_ack(&self) -> u32 {
        self.ack_counter.fetch_add(1, Ordering::Relaxed)
    }

    pub fn recv(&self, socket: &UdpSocket) -> anyhow::Result<Packet> {
        let mut buf = [0; 10000];

        let (n, _addr) = socket.recv_from(&mut buf)?;
        let data = &buf[..n];

        // tracing::debug!("[*] Received {n} bytes from {addr}");

        let in_packet = Packet::from_raw(data)?;

        Ok(in_packet)
    }

    pub fn send(&self, socket: &UdpSocket, content: PacketContent) -> anyhow::Result<()> {
        #[allow(clippy::cast_possible_truncation)]
        let packet = Packet {
            timestamp: SystemTime::now().duration_since(self.est)?.as_micros() as u32,
            dest_socket_id: self.peer_srt_socket_id,
            content,
        };

        socket.send_to(&packet.to_raw(), self.addr)?;

        Ok(())
    }
}

pub fn connect(callback: &dyn Fn(&[u8])) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9000")?;

    let conn = Connection::listen_v5(&socket)?;
    let mut ack_count = 1;

    tracing::info!("Stream started");
    tracing::info!("Stream ID: {:?}", conn.stream_id);

    loop {
        let packet = conn.recv(&socket)?;

        match packet.content {
            PacketContent::Control(ControlPacketInfo::Shutdown) => {
                tracing::info!("Shutdown");
                break;
            }
            PacketContent::Control(control) => {
                tracing::debug!("IN: Control: {control:?}");
            }
            PacketContent::Data(data) => {
                // println!("{data:?}");
                tracing::debug!(
                    "IN: Data {{ packet_sequence_number: {:?}, position: {:?}, order: {:?}, encryption: {:?}, retransmitted: {:?}, message_number: {:?}, length: {:?} }}",
                    data.packet_sequence_number,
                    data.position,
                    data.order,
                    data.encryption,
                    data.retransmitted,
                    data.message_number,
                    data.content.len()
                );
                // println!(" => Payload: {:x?}", data.content);
                let ack = make_ack(&data, ack_count)?;
                tracing::debug!("OUT: {ack:?}");
                ack_count += 1;
                conn.send(&socket, ack)?;

                // [8..]
                let mpeg_packet = &data.content[..];

                callback(mpeg_packet);
            }
        }
    }

    Ok(())
}
