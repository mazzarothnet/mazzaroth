use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub amount: u128,
}