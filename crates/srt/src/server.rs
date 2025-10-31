use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
};

use crate::{
    connection::Connection,
    ops::{handshake_v5, make_ack},
    packet::{Packet, PacketContent, control::ControlPacketInfo},
};

pub struct Server {
    socket: UdpSocket,
    connections: HashMap<SocketAddr, Connection>,
}

impl Server {
    pub fn new() -> anyhow::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:9000")?;

        Ok(Self {
            socket,
            connections: HashMap::new(),
        })
    }

    fn handle(&self, socket: &UdpSocket, conn: &Connection, pack: &Packet) -> anyhow::Result<()> {
        match &pack.content {
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

                let ack_n = conn.inc_ack();
                let ack = make_ack(&data, ack_n)?;
                tracing::debug!("OUT: {ack:?}");
                conn.send(&socket, ack)?;

                // [8..]
                let mpeg_packet = &data.content[..];

                // callback(mpeg_packet);
            }
        }

        Ok(())
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let mut connections = HashMap::new();
        let socket = UdpSocket::bind("0.0.0.0:9000")?;

        loop {
            let mut buf = [0; 10000];

            let (n, addr) = socket.recv_from(&mut buf)?;
            let data = &buf[..n];

            if let Some(conn) = connections.get(&addr) {
                let pack = Packet::from_raw(data)?;

                if matches!(
                    pack.content,
                    PacketContent::Control(ControlPacketInfo::Shutdown)
                ) {
                    connections.remove(&addr);
                    continue;
                }

                Self::handle(&self, &socket, conn, &pack)?;
            } else {
                let Ok(conn) = handshake_v5(&socket) else {
                    continue;
                };
                tracing::info!(conn.stream_id);
                connections.insert(addr, conn);
            }
        }
    }
}
