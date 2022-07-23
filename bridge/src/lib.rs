use ed25519_compact::{PublicKey, Signature};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, ext_contract, near_bindgen, require, AccountId, Promise, PromiseOrValue};

use std::collections::HashMap;

#[ext_contract(ext_xp_nft)]
pub trait ExtXpNft {
    fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Promise;

    fn nft_burn(&mut self, token_id: TokenId, token_owner_id: AccountId) -> Promise;
}

#[ext_contract(ext_nft)]
pub trait ExtNft {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Promise;
}

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
    token_id: TokenId,
    owner_id: AccountId,
    token_metadata: TokenMetadata,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnfreezeNftData {
    action_id: u128,
    token_id: TokenId,
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

    /// Ed25519 Signature verification logic.
    /// Signature check for bridge actions.
    /// Consumes the passed action_id.
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
    pub fn validate_withdraw_fees(
        &mut self,
        data: WithdrawFeeData,
        sig_data: Vec<u8>,
    ) -> PromiseOrValue<()> {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        let account_id: AccountId = data.account_id.parse().unwrap();
        let public_key = near_sdk::PublicKey::try_from(data.public_key).unwrap();
        // Creating new account. It still can fail (e.g. account already exists or name is invalid),
        // but we don't care, we'll get a refund back.
        Promise::new(account_id)
            .create_account()
            .transfer(env::account_balance() - 10_000_000)
            .add_full_access_key(public_key)
            .into()
    }

    #[payable]
    pub fn validate_update_group_key(&mut self, data: UpdateGroupkeyData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        self.group_key = data.group_key;
    }

    /// Transfer foreign NFT. mint wrapped NFT
    #[payable]
    pub fn validate_transfer_nft(&mut self, data: TransferNftData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        ext_xp_nft::ext(env::current_account_id()).nft_mint(
            data.token_id,
            data.owner_id,
            data.token_metadata,
        );
    }

    /// Withdraw foreign NFT
    /// WARN: Even though this contract doesn't check if the burner is trusted,
    /// we check this in the bridge infrastructure(i.e in the validator)
    #[payable]
    pub fn withdraw_nft(&mut self, token_id: TokenId, chain_nonce: u8, to: String, amt: u128) {
        require!(!self.paused, "paused");

        ext_xp_nft::ext(env::current_account_id())
            .nft_burn(token_id, env::predecessor_account_id());

        Promise::new(env::current_account_id()).transfer(amt);

        self.action_cnt += 1;

        env::log_str(format!("chain_nonce: {}", chain_nonce).as_str());
        env::log_str(format!("to: {}", to).as_str());
        env::log_str(format!("nft_contract: {}", self.action_cnt).as_str());
        env::log_str(format!("action_id: {}", self.action_cnt).as_str());
        env::log_str(format!("yocto: {}", amt).as_str());
    }

    /// Freeze NEP-171 token.
    #[payable]
    pub fn freeze_nft(
        &mut self,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        amt: u128,
        mint_with: String,
    ) {
        require!(!self.paused, "paused");

        ext_nft::ext(env::predecessor_account_id()).nft_transfer(
            env::current_account_id(),
            token_id,
            None,
            None,
        );

        Promise::new(env::current_account_id()).transfer(amt);

        self.action_cnt += 1;

        env::log_str(format!("chain_nonce: {}", chain_nonce).as_str());
        env::log_str(format!("to: {}", to).as_str());
        env::log_str(format!("mint_with: {}", mint_with).as_str());
        env::log_str(format!("nft_contract: {}", self.action_cnt).as_str());
        env::log_str(format!("action_id: {}", self.action_cnt).as_str());
        env::log_str(format!("yocto: {}", amt).as_str());
    }

    /// Unfreeze NEP-171 token.
    #[payable]
    pub fn validate_unfreeze_nft(&mut self, data: UnfreezeNftData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(data.action_id, data.try_to_vec().unwrap(), sig_data);

        ext_nft::ext(env::current_account_id()).nft_transfer(
            env::signer_account_id(),
            data.token_id,
            None,
            None,
        );
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
    use near_sdk::{testing_env, Balance};
    use rand_core::OsRng;

    use super::*;

    const NEAR: u128 = 1000000000000000000000000;

    fn set_context(predecessor: &str, amount: Balance) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(amount);
        testing_env!(builder.build());
    }

    fn init_func(group_key: [u8; 32]) -> XpBridge {
        XpBridge::new(group_key)
    }

    #[test]
    fn init() {
        set_context("admin", 1 * NEAR);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let contract = init_func(group_key);

        assert_eq!(group_key, contract.get_group_key());
    }

    #[test]
    fn pause_unpause() {
        set_context("admin", 1 * NEAR);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let mut contract = init_func(group_key);

        let data = PauseData { action_id: 1 };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_pause(data, sig.to_bytes().to_vec());

        assert_eq!(true, contract.is_paused());

        let data = UnpauseData { action_id: 2 };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_unpause(data, sig.to_bytes().to_vec());

        assert_eq!(false, contract.is_paused());
    }

    #[test]
    fn update_group_key() {
        set_context("admin", 1 * NEAR);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let mut contract = init_func(group_key);

        let mut new_csprng = OsRng {};
        let new_kp = Keypair::generate(&mut new_csprng);
        let new_group_key: [u8; 32] = new_kp.public.to_bytes();

        let data = UpdateGroupkeyData {
            action_id: 3,
            group_key: new_group_key,
        };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_update_group_key(data, sig.to_bytes().to_vec());

        assert_eq!(new_group_key, contract.get_group_key());
    }

    #[test]
    fn freeze_unfreeze_nft() {
        set_context("sender", 2 * NEAR);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let mut contract = init_func(group_key);

        contract.freeze_nft(
            "test_nft".to_string(),
            0x0,
            "address_of_foreign".to_string(),
            1 * NEAR,
            "foreign_nft_contract".to_string(),
        );

        let data = UnfreezeNftData {
            action_id: 1,
            token_id: "test_nft".to_string(),
        };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_unfreeze_nft(data, sig.to_bytes().to_vec());
    }

    #[test]
    fn mint_burn_nft() {
        set_context("sender", 2 * NEAR);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let group_key: [u8; 32] = kp.public.to_bytes();

        let mut contract = init_func(group_key);

        let data = TransferNftData {
            action_id: 1,
            token_id: "test_nft".to_string(),
            owner_id: env::predecessor_account_id(),
            token_metadata: TokenMetadata {
                title: Some("title".to_string()),
                description: Some("description".to_string()),
                media: None,
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None,
            },
        };
        let secret: ExpandedSecretKey = (&kp.secret).into();
        let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

        contract.validate_transfer_nft(data, sig.to_bytes().to_vec());

        contract.withdraw_nft(
            "test_nft".to_string(),
            0x0,
            "address_of_foreign".to_string(),
            1 * NEAR,
        );
    }
}
