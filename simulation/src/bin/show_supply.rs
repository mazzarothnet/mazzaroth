use consensus::{
    BEGIN_BLOCK_REWARD, BLOCK_GAS_LIMIT, HALF_BLOCK_REWARD_SIZE, STO_ACCOUNT_MIN_BALANCE,
    STO_BYTES_PER_ACCOUNT, STO_WEI_PER_BYTE, TRANSFER_GAS, get_now_block_reward,
};

fn main() {
    let total_supply = u128::from(HALF_BLOCK_REWARD_SIZE) * 2 * BEGIN_BLOCK_REWARD;
    println!("total_storage {}MAZ", total_supply);
    println!("total u128max {}", u128::MAX);

    let total_size = total_supply / STO_WEI_PER_BYTE;
    println!("total_size {}", total_size);
    let total_storage = total_size / 1024u128 / 1024u128 / 1024u128;
    println!("total_storage {} GB", total_storage);

    let bytes_per_block = BEGIN_BLOCK_REWARD / STO_WEI_PER_BYTE;
    println!("bytes_per_block {}", bytes_per_block);

    let account_per_block = BEGIN_BLOCK_REWARD / STO_BYTES_PER_ACCOUNT / STO_WEI_PER_BYTE;
    println!("account_per_block {}", account_per_block);

    let max_account_num = total_size / STO_BYTES_PER_ACCOUNT;
    println!("max_account_num {}", max_account_num);

    let transfer_per_block = BLOCK_GAS_LIMIT as f64 / TRANSFER_GAS as f64;
    println!("transfer_per_block {}", transfer_per_block);

    let mut now_size = 0;
    for i in 0..HALF_BLOCK_REWARD_SIZE {
        let now_reward = get_now_block_reward(now_size);
        if now_reward >= STO_ACCOUNT_MIN_BALANCE {
            println!("can create account: {}", i);
        } else {
            break;
        }
        now_size += HALF_BLOCK_REWARD_SIZE;
    }

    println!("STO_ACCOUNT_MIN_BALANCE {}", STO_ACCOUNT_MIN_BALANCE);
}
/*
#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Transfer {
    pub inner: TransferInner,
    pub from_signature: Signature,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct TransferInner {
    pub from: AccountKey,
    pub to: AccountKey,
    pub amount: u128,
    pub from_last_action_hash: Hash,
    pub gas_price: u128,
}
*/
