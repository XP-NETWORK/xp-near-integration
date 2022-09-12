use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferNftEvent {
    pub chain_nonce: u8,
    pub to: String,
    pub mint_with: String,
    pub action_id: u128,
    pub amt: u128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnfreezeNftEvent {
    pub chain_nonce: u8,
    pub to: String,
    pub action_id: u128,
    pub amt: u128,
}
