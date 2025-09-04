use alloy_rlp::Encodable;
use consensus::types::AccountKey;
use mvm::core::merkle_tree::TreeKey;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn main() {
    let mut rng = StdRng::seed_from_u64(11121291);
    let mut account_vec: [u8; 33] = [0; 33];
    for i in &mut account_vec {
        *i = rng.random_range(0..255);
    }
    let account_key = AccountKey(account_vec);
    println!("account_key: {:?}", account_key);
    let mut rlp_vec = Vec::new();
    account_key.encode(&mut rlp_vec);
    println!("rlp_vec: {:?}", rlp_vec.len());
    let tree_key = TreeKey {
        mask_num: 0,
        key: account_key,
    };
    let mut rlp_vec = Vec::new();
    tree_key.encode(&mut rlp_vec);
    println!("rlp_vec: {:?}", rlp_vec.len());
}
