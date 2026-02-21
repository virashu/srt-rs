use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

use anyhow::Result;

use crate::{
    connection::Connection,
    constants::MAX_PACKET_SIZE,
    packet::{Packet, PacketContent, control::ControlPacketInfo},
};

type OnConnectHandler = dyn Fn(&Connection);
type OnDiscnnectHandler = dyn Fn(&Connection);
pub type OnDataHandler = dyn Fn(&Connection, &[u8]);

pub struct Server {
    on_connect: Option<Box<OnConnectHandler>>,
    on_disconnect: Option<Box<OnDiscnnectHandler>>,
    on_data: Option<Box<OnDataHandler>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            on_connect: None,
            on_disconnect: None,
            on_data: None,
        }
    }

    pub fn on_connect(&mut self, f: impl Fn(&Connection) + 'static) {
        self.on_connect = Some(Box::new(f));
    }

    pub fn on_disconnect(&mut self, f: impl Fn(&Connection) + 'static) {
        self.on_disconnect = Some(Box::new(f));
    }

    pub fn on_data(&mut self, f: impl Fn(&Connection, &[u8]) + 'static) {
        self.on_data = Some(Box::new(f));
    }

    fn recv(&self, socket: &UdpSocket) -> Result<(SocketAddr, Packet)> {
        let mut buf = [0; MAX_PACKET_SIZE];

        let (n, addr) = socket.recv_from(&mut buf)?;
        let data = &buf[..n];
        let pack = Packet::from_raw(data)?;

        Ok((addr, pack))
    }

    pub fn run(&self, addr: impl ToSocketAddrs) -> Result<()> {
        let socket = UdpSocket::bind(addr)?;

        let mut connections = HashMap::<SocketAddr, Connection>::new();

        loop {
            let (addr, pack) = self.recv(&socket)?;

            // Disconnect
            if matches!(
                pack.content,
                PacketContent::Control(ControlPacketInfo::Shutdown)
            ) {
                if let Some(conn) = connections.remove(&addr) {
                    conn.handle(&pack)?;

                    if let Some(callback) = &self.on_disconnect {
                        callback(&conn);
                    }
                }
            }
            // Handle data
            else if let Some(conn) = connections.get(&addr) {
                conn.handle(&pack)?;
            }
            // Connect
            else if let Ok(conn) = Connection::establish_v5(&socket, self.on_data.as_deref()) {
                if let Some(callback) = &self.on_connect {
                    callback(&conn);
                }
                connections.insert(addr, conn);
            }
        }
    }
}
