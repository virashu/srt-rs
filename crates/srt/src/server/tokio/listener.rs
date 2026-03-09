use std::{
    collections::{BTreeMap, btree_map::Entry},
    net::SocketAddr,
    sync::Arc,
};

use anyhow::{Result, bail};
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    sync::{
        RwLock,
        mpsc::{Receiver, Sender, channel, error::TryRecvError},
    },
    task,
};

use super::connection::AsyncConnection;
use crate::protocol::packet::{Packet, PacketContent, control::ControlPacketInfo};

const MAX_PACKET_SIZE: usize = 1500;
const MAX_CONSECUTIVE_PACKETS_PER_CONNECTION: usize = 5;

pub struct Stream {
    pub(crate) inbound: Receiver<Packet>,
    pub(crate) outbound: Sender<Packet>,
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

    async fn handle_single_connection_outbound(
        socket: &UdpSocket,
        addr: &SocketAddr,
        chan: &mut Receiver<Packet>,
    ) -> Result<()> {
        for _ in 0..MAX_CONSECUTIVE_PACKETS_PER_CONNECTION {
            match chan.try_recv() {
                Ok(pack) => {
                    socket.send_to(&pack.to_raw(), addr).await?;
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(e) => {
                    bail!("Error: {e}");
                }
            }
        }

        Ok(())
    }

    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        let mut inbound: BTreeMap<SocketAddr, Sender<Packet>> = BTreeMap::new();
        let outbound: Arc<RwLock<BTreeMap<SocketAddr, Receiver<Packet>>>> =
            Arc::new(RwLock::new(BTreeMap::new()));

        let socket = Arc::new(UdpSocket::bind(addr).await?);

        let connection_channel = channel(100);

        // Inbound
        task::spawn({
            let socket = socket.clone();
            let outbound = outbound.clone();

            async move {
                loop {
                    // let addr = socket.peek_sender();
                    let (addr, pack) = Self::recv(&socket).await.unwrap();

                    let entry = inbound.entry(addr);

                    match entry {
                        // New connection
                        Entry::Vacant(vacant_entry) => {
                            let (inbound_tx, inbound_rx) = channel(100);
                            let (outbound_tx, outbound_rx) = channel(100);

                            {
                                let mut outbound_lock = outbound.write().await;
                                outbound_lock.insert(addr, outbound_rx);
                            }

                            inbound_tx.send(pack).await.unwrap();
                            vacant_entry.insert(inbound_tx);

                            let stream = Stream {
                                inbound: inbound_rx,
                                outbound: outbound_tx,
                            };

                            connection_channel.0.send(stream).await.unwrap();
                        }
                        Entry::Occupied(mut occupied_entry) => {
                            if matches!(
                                pack.content,
                                PacketContent::Control(ControlPacketInfo::Shutdown)
                            ) {
                                tracing::info!(?addr, "Disconnect");
                                occupied_entry.remove().send(pack).await.unwrap();
                            } else {
                                occupied_entry.get_mut().send(pack).await.unwrap();
                            }
                        }
                    }
                }
            }
        });

        // Outbound
        task::spawn({
            let socket = socket.clone();
            let outbound = outbound.clone();

            async move {
                loop {
                    let mut outbound_lock = outbound.write().await;

                    for (addr, chan) in outbound_lock.iter_mut() {
                        Self::handle_single_connection_outbound(&socket, addr, chan).await;
                    }
                }
            }
        });

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
