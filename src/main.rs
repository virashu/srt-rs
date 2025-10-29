use std::net::UdpSocket;

use rs_srt::packet::{
    Packet, PacketContent,
    control::{ControlPacket, ControlType},
    handshake::{Handshake, HandshakeType},
};

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9000")?;

    let mut buf = [0; 1024];

    //
    // Induction
    //

    let (n, addr) = socket.recv_from(&mut buf)?;
    println!("Connection from: {addr} (read {n} bytes)");
    let data = &buf[..n];

    let packet = Packet::from_raw(data);
    println!("Header: {packet:#?}");
    let payload = Vec::from(&data[16..]);
    let mut hs = Handshake::from_raw(&payload)?;
    println!("{hs:#?}");
    println!("\n{}\n", "=".repeat(40));

    let head = Packet {
        timestamp: packet.timestamp,
        dest_socket_id: hs.srt_socket_id,
        content: PacketContent::Control(ControlPacket {
            control_type: ControlType::Handshake,
            subtype: 0,
        }),
    };

    hs.version = 5;
    hs.srt_socket_id = 0;
    hs.syn_cookie = 42;
    hs.extension_field = 0x4A17;

    println!("{hs:#?}");
    println!("\n{}\n", "=".repeat(40));

    socket.send_to(&head.to_raw(), addr)?;
    socket.send_to(&hs.to_raw(), addr)?;

    //
    // Conclusion
    //

    loop {
        let (n, addr) = socket.recv_from(&mut buf)?;
        println!("(read {n} bytes)");
        let data = &buf[..n];
        let hs = Handshake::from_raw(&data[16..])?;
        if hs.handshake_type == HandshakeType::Induction {
            println!("Got induction");
        }

        // socket.send_to(&hs.to_raw(), addr)?;
    }

    // println!("Done");

    // Ok(())
}
