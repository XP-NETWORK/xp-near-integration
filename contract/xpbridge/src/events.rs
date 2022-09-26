use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{
    env,
    serde::{Deserialize, Serialize},
    serde_json, AccountId,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferNftEvent {
    pub chain_nonce: u8,
    pub to: String,
    pub mint_with: String,
    pub action_id: u128,
    pub amt: u128,
    pub token_id: TokenId,
    pub contract: AccountId,
}

impl TransferNftEvent {
    fn to_json_string(&self) -> String {
        // Events cannot fail to serialize so fine to panic on error
        serde_json::to_string(self)
            .ok()
            .unwrap_or_else(|| env::abort())
    }

    fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    pub fn emit(self) {
        env::log_str(&self.to_json_event_string());
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnfreezeNftEvent {
    pub chain_nonce: u8,
    pub to: String,
    pub action_id: u128,
    pub amt: u128,
    pub token_id: TokenId,
    pub contract: AccountId,
}

impl UnfreezeNftEvent {
    fn to_json_string(&self) -> String {
        // Events cannot fail to serialize so fine to panic on error
        serde_json::to_string(self)
            .ok()
            .unwrap_or_else(|| env::abort())
    }

    fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    pub fn emit(self) {
        env::log_str(&self.to_json_event_string());
    }
}
