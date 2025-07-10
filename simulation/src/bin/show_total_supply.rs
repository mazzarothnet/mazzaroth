use consensus::{
    BEGIN_BLOCK_REWARD, BYTES_PER_ACCOUNT, EXPECTED_BLOCK_SIZE, HALF_BLOCK_REWARD, MAZ_PER_BYTE,
};
use mvm::models::transfer::Transfer;

fn main() {
    let total_supply = HALF_BLOCK_REWARD as u128 * 2 * BEGIN_BLOCK_REWARD;
    println!("total_storage {}MAZ", total_supply);

    let total_size = total_supply / MAZ_PER_BYTE;
    println!("total_size {}", total_size);
    let total_storage = total_size / 1024u128 / 1024u128 / 1024u128;
    println!("total_storage {} GB", total_storage);

    let bytes_per_block = BEGIN_BLOCK_REWARD / MAZ_PER_BYTE;
    println!("bytes_per_block {}", bytes_per_block);

    let account_per_block = BEGIN_BLOCK_REWARD / BYTES_PER_ACCOUNT / MAZ_PER_BYTE;
    println!("account_per_block {}", account_per_block);

    let max_account_num = total_size / BYTES_PER_ACCOUNT;
    println!("max_account_num {}", max_account_num);

    let transfer_per_block = EXPECTED_BLOCK_SIZE as f64 / (33. + 33. + 16. + 32. + 16. + 32.);
    println!("transfer_per_block {}", transfer_per_block);

    let transfer_sizeof = std::mem::size_of::<Transfer>() as f64;
    println!("transfer_sizeof {}", transfer_sizeof);
    let transfer_per_block_sizeof = EXPECTED_BLOCK_SIZE as f64 / transfer_sizeof;
    println!("transfer_per_block_sizeof {}", transfer_per_block_sizeof);
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
