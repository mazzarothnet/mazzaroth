use mazzarothd::network::gossip::load_or_generate_keypair;

#[allow(clippy::unwrap_used)]
fn main() {
    let path = "mazzaroth_data";
    if !std::path::Path::new(path).exists() {
        std::fs::create_dir_all(path).unwrap();
    }
    let path = format!("{}/p2p_keypair.bin", path);
    let keypair = load_or_generate_keypair(&path).unwrap();
    let peer_id = keypair.public().to_peer_id();
    println!("peer_id: {}", peer_id);
}