use ed25519_compact::{PublicKey, Signature};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::sha256;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, Promise, PromiseError};

use std::collections::HashMap;
pub mod events;
pub mod external;
pub use crate::events::*;
pub use crate::external::*;

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PauseData {
    action_id: U128,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnpauseData {
    action_id: U128,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdateGroupkeyData {
    action_id: U128,
    group_key: [u8; 32],
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WhitelistData {
    action_id: U128,
    token_contract: String,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawFeeData {
    pub action_id: U128,
    pub account_id: String,
    pub public_key: Vec<u8>,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferNftData {
    action_id: U128,
    mint_with: AccountId,
    token_id: TokenId,
    owner_id: AccountId,
    token_metadata: TokenMetadata,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnfreezeNftData {
    action_id: U128,
    token_contract: AccountId,
    token_id: TokenId,
    receiver_id: AccountId,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct XpBridge {
    consumed_actions: HashMap<u128, bool>,
    paused: bool,
    tx_fees: u128,
    group_key: [u8; 32],
    action_cnt: u128,
    whitelist: HashMap<String, bool>,
}

#[near_bindgen]
impl XpBridge {
    #[init]
    pub fn initialize(group_key: [u8; 32]) -> Self {
        assert!(
            env::current_account_id() == env::predecessor_account_id(),
            "Unauthorized"
        );

        Self {
            consumed_actions: HashMap::new(),
            paused: false,
            tx_fees: 0,
            group_key,
            action_cnt: 0,
            whitelist: HashMap::new(),
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
    pub fn validate_pause(&mut self, data: PauseData, sig_data: Base64VecU8) {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            sha256(data.try_to_vec().unwrap().as_slice()),
            sig_data.into(),
        );

        self.paused = true;
    }

    #[payable]
    pub fn validate_unpause(&mut self, data: UnpauseData, sig_data: Base64VecU8) {
        require!(self.paused, "unpaused");

        self.require_sig(
            data.action_id.into(),
            sha256(data.try_to_vec().unwrap().as_slice()),
            sig_data.into(),
        );

        self.paused = false;
    }

    #[payable]
    pub fn validate_withdraw_fees(
        &mut self,
        data: WithdrawFeeData,
        sig_data: Base64VecU8,
    ) -> Promise {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data.into(),
        );

        let storage_used = env::storage_usage();
        let amt = self.tx_fees - storage_used as u128 * env::storage_byte_cost();
        Promise::new(env::current_account_id())
            .transfer(amt)
            .then(Self::ext(env::current_account_id()).withdraw_fee_callback())
    }

    #[private]
    pub fn withdraw_fee_callback(
        &mut self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        require!(call_result.is_ok(), "withdraw failed");

        self.tx_fees = 0;
    }

    #[payable]
    pub fn validate_update_group_key(&mut self, data: UpdateGroupkeyData, sig_data: Base64VecU8) {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            sha256(data.try_to_vec().unwrap().as_slice()),
            sig_data.into(),
        );

        self.group_key = data.group_key;
    }

    #[payable]
    pub fn validate_whitelist(&mut self, data: WhitelistData, sig_data: Base64VecU8) {
        require!(!self.paused, "paused");

        require!(
            !self
                .whitelist
                .contains_key(&data.token_contract.to_string()),
            "Already whitelist"
        );

        self.require_sig(
            data.action_id.into(),
            sha256(data.try_to_vec().unwrap().as_slice()),
            sig_data.into(),
        );

        self.whitelist.insert(data.token_contract, true);
    }

    #[payable]
    pub fn validate_blacklist(&mut self, data: WhitelistData, sig_data: Base64VecU8) {
        require!(!self.paused, "paused");

        require!(
            self.whitelist
                .contains_key(&data.token_contract.to_string()),
            "Not whitelist"
        );

        self.require_sig(
            data.action_id.into(),
            sha256(data.try_to_vec().unwrap().as_slice()),
            sig_data.into(),
        );

        self.whitelist.remove(&data.token_contract);
    }

    /// Transfer foreign NFT. mint wrapped NFT
    #[payable]
    pub fn validate_transfer_nft(
        &mut self,
        data: TransferNftData,
        sig_data: Base64VecU8,
    ) -> Promise {
        require!(!self.paused, "paused");

        require!(
            self.whitelist.contains_key(&data.mint_with.to_string()),
            "Not whitelist"
        );

        self.require_sig(
            data.action_id.into(),
            sha256(data.try_to_vec().unwrap().as_slice()),
            sig_data.into(),
        );

        xpnft::ext(data.mint_with)
            .with_attached_deposit(env::attached_deposit())
            .nft_mint(data.token_id, data.owner_id, data.token_metadata)
    }

    /// Withdraw foreign NFT
    /// WARN: Even though this contract doesn't check if the burner is trusted,
    /// we check this in the bridge infrastructure(i.e in the validator)
    #[payable]
    pub fn withdraw_nft(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        amt: U128,
    ) -> Promise {
        require!(!self.paused, "paused");

        require!(
            self.whitelist
                .contains_key(&token_contract.clone().to_string()),
            "Not whitelist"
        );

        xpnft::ext(token_contract.clone())
            .nft_burn(token_id.clone(), env::predecessor_account_id())
            .then(Self::ext(env::current_account_id()).withdraw_callback(
                token_contract,
                token_id,
                chain_nonce,
                to,
                amt.into(),
            ))
    }

    #[private]
    pub fn withdraw_callback(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        amt: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Promise {
        require!(call_result.is_ok(), "withdraw failed");

        self.action_cnt += 1;
        self.tx_fees += amt;

        UnfreezeNftEvent {
            action_id: self.action_cnt,
            chain_nonce,
            to,
            amt,
            contract: token_contract,
            token_id,
        }
        .emit();

        Promise::new(env::current_account_id()).transfer(amt.into())
    }

    /// Freeze NEP-171 token.
    #[payable]
    pub fn freeze_nft(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        mint_with: String,
        amt: U128,
    ) -> Promise {
        require!(!self.paused, "paused");

        common_nft::ext(token_contract.clone())
            .with_attached_deposit(1)
            .nft_transfer(env::current_account_id(), token_id.clone(), None, None)
            .then(Self::ext(env::current_account_id()).freeze_callback(
                token_contract,
                token_id,
                chain_nonce,
                to,
                mint_with,
                amt.into(),
            ))
    }

    #[private]
    pub fn freeze_callback(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        mint_with: String,
        amt: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Promise {
        require!(call_result.is_ok(), "freeze failed");

        self.action_cnt += 1;
        self.tx_fees += amt;

        TransferNftEvent {
            action_id: self.action_cnt,
            chain_nonce,
            to,
            amt,
            contract: token_contract,
            token_id,
            mint_with,
        }
        .emit();

        Promise::new(env::current_account_id()).transfer(amt.into())
    }

    /// Unfreeze NEP-171 token.
    #[payable]
    pub fn validate_unfreeze_nft(
        &mut self,
        data: UnfreezeNftData,
        sig_data: Base64VecU8,
    ) -> Promise {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data.into(),
        );

        common_nft::ext(data.token_contract).nft_transfer(
            data.receiver_id,
            data.token_id,
            None,
            None,
        )
    }

    pub fn get_group_key(&self) -> [u8; 32] {
        self.group_key
    }

    pub fn is_whitelist(&self, contract_id: String) -> bool {
        self.whitelist.contains_key(&contract_id)
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn get_action_id(&self) -> U128 {
        U128(self.action_cnt)
    }
}
