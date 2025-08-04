use mazzarothd::gossip::{
    channel_block::ChannelBlock,
    ipv6_udp::{Ipv6UdpRecv, Ipv6UdpSend, RecvAction, SendAction},
};
use std::net::UdpSocket;
use utils::{log::init_log, sha256::sha256_hash};

const RECV_ADDR: &str = "[::]:8081";
const SEND_ADDR_RECV: &str = "[::1]:8080";

fn main() {
    init_log();
    let socket = UdpSocket::bind(RECV_ADDR).unwrap();
    let socket_send = socket.try_clone().unwrap();
    let mut udp_recv = Ipv6UdpRecv::new();
    udp_recv.action(RecvAction::AddNode(SEND_ADDR_RECV.parse().unwrap(), 1));
    let mut udp_send = Ipv6UdpSend::new(socket_send);
    udp_send
        .action(SendAction::AddNode(SEND_ADDR_RECV.parse().unwrap()))
        .unwrap();
    let mut count = 0;
    loop {
        println!("recv start {}", count);
        count += 1;
        let mut buf = [0; 65535];
        let (len, src) = socket.recv_from(&mut buf).unwrap();
        let data = &buf[..len];
        let ans = udp_recv.recv(src, data);
        if let Some(ans) = ans {
            let send_data = ChannelBlock {
                topic_id: 0,
                data: "hello".as_bytes().to_vec(),
            };
            udp_send
                .action(SendAction::Broadcast(send_data, None))
                .unwrap();
            let hash = sha256_hash(&ans.data);
            println!("recv hash: {:?}", hash);
        }
    }
}
