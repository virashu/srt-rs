use std::{
    collections::{BTreeMap, btree_map::Entry},
    net::SocketAddr,
    sync::Arc,
};

use anyhow::Result;
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    sync::mpsc::{Receiver, Sender, channel},
};

use crate::protocol::packet::{Packet, PacketContent, control::ControlPacketInfo};

const MAX_PACKET_SIZE: usize = 1500;
const MAX_CONSECUTIVE_PACKETS_PER_CONNECTION: usize = 5;

pub struct Stream {
    addr: SocketAddr,
    inbound: Receiver<Packet>,
    outbound: Sender<(SocketAddr, Packet)>,
}

impl Stream {
    pub async fn recv(&mut self) -> Option<Packet> {
        self.inbound.recv().await
    }

    pub async fn send(&self, pack: Packet) -> Result<()> {
        self.outbound.send((self.addr.clone(), pack)).await?;
        Ok(())
    }
}

pub struct AsyncListener {
    connection_queue: Receiver<Stream>,
}

impl AsyncListener {
    async fn recv(socket: &UdpSocket) -> Result<(SocketAddr, Packet)> {
        let mut buf = [0; MAX_PACKET_SIZE];

        let (n, addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..n];
        let pack = Packet::from_raw(data)?;

        Ok((addr, pack))
    }

    async fn inbound_loop(
        socket: Arc<UdpSocket>,
        connection_channel: Sender<Stream>,
        mut inbound: BTreeMap<SocketAddr, Sender<Packet>>,
        outbound_tx: Sender<(SocketAddr, Packet)>,
    ) -> Result<()> {
        loop {
            // let addr = socket.peek_sender();
            let (addr, pack) = Self::recv(&socket).await?;

            let entry = inbound.entry(addr);

            match entry {
                // New connection
                Entry::Vacant(vacant_entry) => {
                    let (inbound_tx, inbound_rx) = channel(100);

                    inbound_tx.send(pack).await?;
                    vacant_entry.insert(inbound_tx);

                    let stream = Stream {
                        addr,
                        inbound: inbound_rx,
                        outbound: outbound_tx.clone(),
                    };

                    connection_channel.send(stream).await?;
                }

                // Existing connection
                Entry::Occupied(mut occupied_entry) => {
                    if matches!(
                        pack.content,
                        PacketContent::Control(ControlPacketInfo::Shutdown)
                    ) {
                        tracing::info!(?addr, "Disconnect");
                        occupied_entry.remove().send(pack).await?;
                    } else {
                        occupied_entry.get_mut().send(pack).await?;
                    }
                }
            }
        }
    }

    async fn outbound_loop(
        socket: Arc<UdpSocket>,
        mut outbound_rx: Receiver<(SocketAddr, Packet)>,
    ) -> Result<()> {
        while let Some((addr, pack)) = outbound_rx.recv().await {
            socket.send_to(&pack.to_raw(), addr).await?;
        }

        Ok(())
    }

    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        let inbound: BTreeMap<SocketAddr, Sender<Packet>> = BTreeMap::new();
        let (outbound_tx, outbound_rx) = channel::<(SocketAddr, Packet)>(100);

        let socket = Arc::new(UdpSocket::bind(addr).await?);

        let connection_channel = channel(100);

        // Inbound
        tokio::spawn(Self::inbound_loop(
            socket.clone(),
            connection_channel.0,
            inbound,
            outbound_tx.clone(),
        ));

        // Outbound
        tokio::spawn(Self::outbound_loop(socket.clone(), outbound_rx));

        Ok(Self {
            connection_queue: connection_channel.1,
        })
    }

    pub fn incoming(self) -> Incoming {
        Incoming { listener: self }
    }
}

pub struct Incoming {
    listener: AsyncListener,
}

impl Incoming {
    pub async fn poll_next(&mut self) -> Option<Stream> {
        self.listener.connection_queue.recv().await
    }
}

// impl Iterator for Incoming {
//     type Item = impl Future<Output = Option<Stream>>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.listener.connection_queue.recv()
//     }
// }

// impl AsyncIterator for Incoming {
//     type Item = Stream;

//     async fn poll_next(&mut self) -> Option<Self::Item> {
//         if let Some(stream) = self.listener.connection_queue.recv().await {
//             AsyncConnection::establish_v5(stream).await.ok()
//         } else {
//             None
//         }
//     }
// }
