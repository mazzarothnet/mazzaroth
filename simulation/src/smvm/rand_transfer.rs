use consensus::types::{AccountKey, Hash, Signature};
use mvm::models::transfer::{Transfer, TransferInner};
use rand::{Rng, rngs::StdRng};

fn gen_rand_vec<const L: usize>(rng: &mut StdRng) -> [u8; L] {
    let mut vec = [0; L];
    for i in &mut vec {
        *i = rng.random_range(0..255);
    }
    vec
}

pub fn new_rand_transfer(rng: &mut StdRng) -> Transfer {
    let from = AccountKey(gen_rand_vec(rng));
    let to = AccountKey(gen_rand_vec(rng));
    let from_last_action_hash = Hash(gen_rand_vec(rng));
    let amount = rng.random_range(0..1000000000000000000u128);
    Transfer {
        inner: TransferInner {
            from,
            to,
            amount,
            from_last_action_hash,
            gas_price: 0,
        },
        from_signature: Signature(gen_rand_vec(rng)),
    }
}

pub fn gen_rand_transfer(rng: &mut StdRng, num: u64) -> Vec<Transfer> {
    let mut transfers = Vec::new();
    for _ in 0..num {
        transfers.push(new_rand_transfer(rng));
    }
    transfers
}
