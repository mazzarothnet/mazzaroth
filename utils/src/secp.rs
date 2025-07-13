use anyhow::Context;
use secp256k1::{Message, PublicKey, SecretKey, ecdsa::Signature};

pub fn sign_message(message: &[u8; 32], private_key: &[u8; 32]) -> anyhow::Result<[u8; 64]> {
    let context = secp256k1::Secp256k1::new();
    let private_key = SecretKey::from_byte_array(*private_key)?;
    let signature = context.sign_ecdsa(Message::from_digest(*message), &private_key);
    Ok(signature.serialize_compact())
}

pub fn verify_message(
    message: &[u8; 32],
    signature: &[u8; 64],
    public_key: &[u8; 33],
) -> anyhow::Result<()> {
    let context = secp256k1::Secp256k1::new();
    let public_key = PublicKey::from_byte_array_compressed(*public_key)?;
    let signature = Signature::from_compact(signature)?;
    context
        .verify_ecdsa(Message::from_digest(*message), &signature, &public_key)
        .with_context(|| "verify message failed")
}
