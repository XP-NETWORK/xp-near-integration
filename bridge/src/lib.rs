use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};

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
    #[init]
    pub fn new(group_key: [u8; 32]) -> Self {
        Self {
            consumed_actions: HashMap::new(),
            paused: false,
            tx_fees: 0,
            group_key,
            action_cnt: 0,
        }
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

    pub fn get_group_key(&self) -> [u8; 32] {
        self.group_key
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use ed25519_dalek::Keypair;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use rand_core::OsRng;
    use std::collections::HashMap;

    use super::*;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("alice".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_bridge() {
        let context = get_context(false);
        testing_env!(context);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let contract = XpBridge::new(group_key);

        let context = get_context(true);
        testing_env!(context);

        assert_eq!(group_key, contract.get_group_key());
    }
}
