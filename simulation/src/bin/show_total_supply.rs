use consensus::{
    BEGIN_BLOCK_REWARD, BLOCK_GAS_LIMIT, HALF_BLOCK_REWARD,
    STO_BYTES_PER_ACCOUNT, STO_WEI_PER_BYTE, TRANSFER_GAS,
};

fn main() {
    let total_supply = HALF_BLOCK_REWARD as u128 * 2 * BEGIN_BLOCK_REWARD;
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
    pub from_last_action_hash: ActionHash,
    pub gas_price: u128,
}
*/
