use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;

use std::collections::HashMap;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct XpBridge {
    consumed_actions: HashMap<u128, bool>,
    paused: bool,
    tx_fees: u128,
    group_key: [u8; 32],
    action_cnt: u128,
}

#[near_bindgen]
impl XpBridge {
    #[payable]
    pub fn initialize(&mut self, group_key: [u8; 32]) {
        // let account_id = env::signer_account_id();
        self.paused = false;
        self.tx_fees = 0;
        self.group_key = group_key;
        self.action_cnt = 0;
    }

    #[payable]
    pub fn validate_pause(&mut self) {
        // TODO:
        self.paused = true;
    }

    #[payable]
    pub fn validate_unpause(&mut self) {
        // TODO:
        self.paused = false;
    }

    #[payable]
    pub fn validate_withdraw_fees(&mut self) {
        // TODO:
    }

    #[payable]
    pub fn validate_update_group_key(&mut self, group_key: [u8; 32]) {
        // TODO:
        self.group_key = group_key;
    }

    #[payable]
    pub fn validate_transfer_nft(&mut self) {
        // TODO:
    }

    #[payable]
    pub fn withdraw_nft(&mut self) {
        // TODO:
    }

    #[payable]
    pub fn freeze_nft(&mut self) {
        // TODO:
    }

    #[payable]
    pub fn validate_unfreeze_nft(&mut self) {
        // TODO:
    }
}
