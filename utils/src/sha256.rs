use sha2::Digest;

pub fn sha256_hash_rlp<T: alloy_rlp::Encodable>(t: &T) -> [u8; 32] {
    let mut msg = Vec::new();
    alloy_rlp::Encodable::encode(t, &mut msg);
    sha256_hash(&msg)
}

pub fn sha256_hash(msg: &[u8]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(msg);
    hasher.finalize().into()
}
