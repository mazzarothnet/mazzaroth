// src/bin/client.rs

use mazzarothd::gossip::ipv6_udp::{Ipv6UdpRecv, Ipv6UdpSend, RecvAction, SendAction};
use std::{
    fs::File,
    io::Read,
    net::UdpSocket,
    time::{Duration, Instant},
};
use utils::sha256::sha256_hash;
const SEND_ADDR: &str = "[::]:8080";
const RECV_ADDR_RECV: &str = "[::1]:8081";

fn main() {
    //init_log();
    let socket_send = UdpSocket::bind(SEND_ADDR).unwrap();
    let socket_recv = socket_send.try_clone().unwrap();
    let mut file = File::open(
        "test_block/block_0000000000000000000000000000000000000000000000000000000000000000.rlp",
    )
    .unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let hash = sha256_hash(&buf);

    println!("send hash: {:?}", hash);
    let mut udp_recv = Ipv6UdpRecv::new();
    udp_recv.action(RecvAction::AddNode(RECV_ADDR_RECV.parse().unwrap(), 1));

    let mut udp_send = Ipv6UdpSend::new(socket_send.try_clone().unwrap());
    udp_send
        .action(SendAction::AddNode(RECV_ADDR_RECV.parse().unwrap()))
        .unwrap();
    loop {
        let now = Instant::now();
        udp_send
            .action(SendAction::Broadcast(
                mazzarothd::gossip::channel_block::ChannelBlock {
                    topic_id: 0,
                    data: buf.clone(),
                },
                None,
            ))
            .unwrap();
        println!("send success");
        loop {
            println!("recv start");
            let mut buf = [0; 65535];
            let (len, src) = socket_recv.recv_from(&mut buf).unwrap();
            let data = &buf[..len];
            let ans = udp_recv.recv(src, data);
            if let Some(ans) = ans {
                let cast_time = now.elapsed();
                let str = String::from_utf8(ans.data).unwrap();
                println!("recv: {} time: {:?}", str, cast_time.as_millis());
                break;
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}
