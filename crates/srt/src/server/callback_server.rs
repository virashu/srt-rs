use std::{
    collections::{HashMap, hash_map::Entry},
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

use anyhow::Result;

use super::callback_connection::CallbackConnection;
use crate::protocol::{
    constants::MAX_PACKET_SIZE,
    packet::{Packet, PacketContent, control::ControlPacketInfo},
};

type OnConnectHandler = dyn Fn(&CallbackConnection);
type OnDiscnnectHandler = dyn Fn(&CallbackConnection);
pub type OnDataHandler = dyn Fn(&CallbackConnection, &[u8]);

pub struct CallbackServer {
    on_connect: Option<Box<OnConnectHandler>>,
    on_disconnect: Option<Box<OnDiscnnectHandler>>,
    on_data: Option<Box<OnDataHandler>>,
}

impl CallbackServer {
    pub fn new() -> Self {
        Self {
            on_connect: None,
            on_disconnect: None,
            on_data: None,
        }
    }

    pub fn on_connect(&mut self, f: impl Fn(&CallbackConnection) + 'static) {
        self.on_connect = Some(Box::new(f));
    }

    pub fn on_disconnect(&mut self, f: impl Fn(&CallbackConnection) + 'static) {
        self.on_disconnect = Some(Box::new(f));
    }

    pub fn on_data(&mut self, f: impl Fn(&CallbackConnection, &[u8]) + 'static) {
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
        let _span = tracing::info_span!("srt_server").entered();

        let socket = UdpSocket::bind(addr)?;

        let mut connections = HashMap::<SocketAddr, CallbackConnection>::new();

        loop {
            let (addr, pack) = self.recv(&socket)?;
            let entry = connections.entry(addr);

            match entry {
                // New connection
                Entry::Vacant(vacant_entry) => {
                    match CallbackConnection::establish_v5(&socket, self.on_data.as_deref()) {
                        Ok(conn) => {
                            tracing::info!(?addr, "New connection");
                            let conn = vacant_entry.insert(conn);
                            self.on_connect.as_ref().inspect(|f| f(conn));
                        }
                        Err(e) => tracing::error!("Failed to establish connection: {e}"),
                    }
                }
                // Existing connection
                Entry::Occupied(occupied_entry) => {
                    occupied_entry.get().handle(&pack)?;

                    if matches!(
                        pack.content,
                        PacketContent::Control(ControlPacketInfo::Shutdown)
                    ) {
                        tracing::info!(?addr, "Disconnect");
                        let conn = occupied_entry.remove();
                        self.on_disconnect.as_ref().inspect(|f| f(&conn));
                    }
                }
            }
        }
    }
}
