use anyhow::Context;
use consensus::types::AccountKey;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use utils::{
    file::{read_from_json, write_to_json},
    secp::gen_keypair,
    time::get_current_time_ms,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountKeyPair {
    pub public_key: AccountKey,
    pub private_key: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountManager {
    pub account_map: Vec<AccountKeyPair>,
    pub now_selected_account: AccountKey,
}

impl Default for AccountManager {
    fn default() -> Self {
        let now_time = get_current_time_ms();
        let mut rng = rand::rngs::StdRng::seed_from_u64(now_time as u64);
        let (secret_key, public_key) = gen_keypair(&mut rng);
        let account_key = AccountKey(public_key);
        let mut account_map = Vec::new();
        account_map.push(
            AccountKeyPair {
                public_key: account_key.clone(),
                private_key: secret_key,
            },
        );
        Self {
            account_map,
            now_selected_account: account_key,
        }
    }
}

impl AccountManager {
    pub fn init(path: &str) -> anyhow::Result<Self> {
        let os_path = Path::new(path);
        if !os_path.exists() {
            let account_manager = Self::default();
            write_to_json(path, &account_manager)
                .with_context(|| "Failed to write account manager file")?;
        }
        let account_manager =
            read_from_json(path).with_context(|| "Failed to read account manager file")?;
        Ok(account_manager)
    }
}
