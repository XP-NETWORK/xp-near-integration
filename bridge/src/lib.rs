use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, Promise, log, require, PromiseOrValue, AccountId};
use ed25519_compact::{PublicKey, Signature};

use std::collections::HashMap;

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PauseData {
    action_id: u128,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnpauseData {
    action_id: u128,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdateGroupkeyData {
    action_id: u128,
    group_key: [u8; 32],
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawFeeData {
    pub action_id: u128,
    pub account_id: String,
    pub public_key: Vec<u8>,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferNftData {
    action_id: u128,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnfreezeNftData {
    action_id: u128,
}

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

    #[private]
    fn require_sig(&mut self, action_id: u128, data: Vec<u8>, sig_data: Vec<u8>) {
        let f = self.consumed_actions.contains_key(&action_id);
        require!(!f, "Duplicated Action");

        self.consumed_actions.insert(action_id, true);

        let sig = Signature::new(sig_data.as_slice().try_into().unwrap());
        let key = PublicKey::new(self.group_key);
        let res = key.verify(data, &sig);
        require!(res.is_ok(), "Unauthorized Action");
    }

    #[payable]
    pub fn validate_pause(&mut self, data: PauseData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        self.paused = true;
    }

    #[payable]
    pub fn validate_unpause(&mut self, data: UnpauseData, sig_data: Vec<u8>) {
        require!(self.paused, "unpaused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        self.paused = false;
    }

    #[payable]
    pub fn validate_withdraw_fees(&mut self, data: WithdrawFeeData, sig_data: Vec<u8>) -> PromiseOrValue<()> {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);
        
        
        let account_id: AccountId = data.account_id.parse().unwrap();
        let public_key = near_sdk::PublicKey::try_from(data.public_key).unwrap();
        // Creating new account. It still can fail (e.g. account already exists or name is invalid),
        // but we don't care, we'll get a refund back.
        Promise::new(account_id)
            .create_account()
            .transfer(env::account_balance() / 1000)
            .add_full_access_key(public_key)
            .into()
    }

    #[payable]
    pub fn validate_update_group_key(&mut self, data: UpdateGroupkeyData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");
        
        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        self.group_key = data.group_key;
    }

    #[payable]
    pub fn validate_transfer_nft(&mut self, data: TransferNftData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        // TODO:
    }

    #[payable]
    pub fn withdraw_nft(&mut self) {
        require!(!self.paused, "paused");
        // TODO:
    }

    #[payable]
    pub fn freeze_nft(&mut self) {
        require!(!self.paused, "paused");
        // TODO:
    }

    #[payable]
    pub fn validate_unfreeze_nft(&mut self, data: UnfreezeNftData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);
        // TODO:
    }

    pub fn get_group_key(&self) -> [u8; 32] {
        self.group_key
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use ed25519_dalek::{ExpandedSecretKey, Keypair};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use rand_core::OsRng;

    use super::*;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("alice".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    fn init_func(group_key: [u8; 32]) -> XpBridge {
        XpBridge::new(group_key)
    }

    #[test]
    fn test_init() {
        let context = get_context(false);
        testing_env!(context);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let contract = init_func(group_key);

        let context = get_context(true);
        testing_env!(context);

        assert_eq!(group_key, contract.get_group_key());
    }

    #[test]
    fn test_pause_unpause() {
        let context = get_context(false);
        testing_env!(context);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let mut contract = init_func(group_key);

        let data = PauseData { action_id: 1 };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_pause(data, sig.to_bytes().to_vec());

        let context = get_context(true);
        testing_env!(context);

        assert_eq!(true, contract.is_paused());

        let data = UnpauseData { action_id: 2 };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_unpause(data, sig.to_bytes().to_vec());

        let context = get_context(true);
        testing_env!(context);

        assert_eq!(false, contract.is_paused());
    }

    #[test]
    fn test_update_group_key() {
        let context = get_context(false);
        testing_env!(context);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let mut contract = init_func(group_key);

        let mut new_csprng = OsRng {};
        let new_kp = Keypair::generate(&mut new_csprng);
        let new_group_key: [u8; 32] = new_kp.public.to_bytes();

        let data = UpdateGroupkeyData { action_id: 3, group_key: new_group_key };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_update_group_key(data, sig.to_bytes().to_vec());

        let context = get_context(true);
        testing_env!(context);

        assert_eq!(new_group_key, contract.get_group_key());
    }
}
