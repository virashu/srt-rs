use rs_srt::{
    ops::{handshake_v5, make_ack},
    packet::{Packet, PacketContent, control::ControlPacketInfo},
};

use std::net::{SocketAddr, UdpSocket};

fn get_packet(socket: &UdpSocket) -> anyhow::Result<(Packet, SocketAddr)> {
    let mut buf = [0; 10000];

    let (n, addr) = socket.recv_from(&mut buf).unwrap();
    let data = &buf[..n];

    println!("[*] Received {n} bytes from {addr}");

    let in_packet = Packet::from_raw(data)?;

    Ok((in_packet, addr))
}

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9000")?;

    loop {
        let conn = handshake_v5(&socket)?;
        let mut ack_count = 1;

        println!("{}\nStream started\n{}", "=".repeat(14), "=".repeat(14));

        loop {
            let (packet, addr) = get_packet(&socket)?;
            println!("  {:?}\n", packet.content);

            if matches!(
                packet.content,
                PacketContent::Control(ControlPacketInfo::Shutdown)
            ) {
                println!("{}\nShutdown\n{}", "=".repeat(14), "=".repeat(14));
                break;
            }

            if let PacketContent::Data(data) = packet.content {
                let ack = make_ack(&data, ack_count)?;
                ack_count += 1;
                conn.send(&socket, ack)?;
            }
        }
    }
}
