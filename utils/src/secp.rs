use anyhow::Context;
use rand::rngs::StdRng;
use secp256k1::{Message, PublicKey, SecretKey, ecdsa::Signature};

pub fn gen_keypair(rng: &mut StdRng) -> ([u8; 32], [u8; 33]) {
    let (secret_key, public_key) = secp256k1::SECP256K1.generate_keypair(rng);
    (secret_key.secret_bytes(), public_key.serialize())
}

pub fn sign_message(message: &[u8; 32], private_key: &[u8; 32]) -> anyhow::Result<[u8; 64]> {
    let private_key =
        SecretKey::from_byte_array(*private_key).with_context(|| "invalid private key")?;
    let signature = secp256k1::SECP256K1.sign_ecdsa(Message::from_digest(*message), &private_key);
    Ok(signature.serialize_compact())
}

pub fn verify_message(
    message: &[u8; 32],
    signature: &[u8; 64],
    public_key: &[u8; 33],
) -> anyhow::Result<()> {
    let public_key = PublicKey::from_byte_array_compressed(*public_key)?;
    let signature = Signature::from_compact(signature)?;
    secp256k1::SECP256K1
        .verify_ecdsa(Message::from_digest(*message), &signature, &public_key)
        .with_context(|| "verify message failed")
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::sha256::sha256_hash;

    use super::*;

    #[test]
    fn test_sign_message() {
        let message = sha256_hash(b"hello");
        let (private_key, public_key) = secp256k1::SECP256K1.generate_keypair(&mut rand::rng());
        let private_key_bytes = private_key.secret_bytes();
        let public_key_bytes = public_key.serialize();
        let signature = sign_message(&message, &private_key_bytes).unwrap();
        assert!(verify_message(&message, &signature, &public_key_bytes).is_ok());
    }
}
