use log::info;

use crate::gossip::{
    channel_block::ChannelBlock,
    proto::{PING_TOPIC, PONG_TOPIC, REQ_LISTEN_LIST_TOPIC},
    udp::SendAction,
    worker::GossipBlock,
};
use std::{collections::HashMap, net::SocketAddr};

pub fn process_gossip_block(
    gossip_block: GossipBlock,
    listen_list: &mut HashMap<SocketAddr, u32>,
) -> anyhow::Result<Vec<SendAction>> {
    let mut send_actions = Vec::new();
    match gossip_block.data.topic_id {
        PING_TOPIC => {
            info!("PING from {}", gossip_block.src);
            let addr = gossip_block.src.to_string();
            let send_action = SendAction::Send(
                ChannelBlock {
                    topic_id: PONG_TOPIC,
                    data: addr.as_bytes().to_vec(),
                },
                gossip_block.src,
            );
            send_actions.push(send_action);
        }
        PONG_TOPIC => {
            let from_addr = gossip_block.src;
            let me_addr = String::from_utf8(gossip_block.data.data)?;
            info!("PONG from {} to {}", from_addr, me_addr);
            if listen_list.is_empty() {
                send_actions.push(SendAction::Send(
                    ChannelBlock {
                        topic_id: REQ_LISTEN_LIST_TOPIC,
                        data: Vec::new(),
                    },
                    from_addr,
                ))
            }
        }
        REQ_LISTEN_LIST_TOPIC => {
            info!("REQ_LISTEN_LIST_TOPIC from {}", gossip_block.src);
        }
        _ => {}
    };

    Ok(send_actions)
}
