use std::sync::Mutex;
use crate::gossip::ipv6_udp::{Ipv6UdpRecv, Ipv6UdpSend};

pub mod channel_block;
/// @ fawkes There are tow point can be optimized:
/// 1. send many udp packet at once
/// 2. optimize the reed solomon algorithm
pub mod ipv6_udp;
pub mod proto;
pub mod worker;

lazy_static::lazy_static! {
    pub static ref UDP_RECV: Mutex<Ipv6UdpRecv> = Mutex::new(Ipv6UdpRecv::new());
    pub static ref UDP_SEND: Mutex<Ipv6UdpSend> = Mutex::new(Ipv6UdpSend::new());
}