use ed25519_compact::{PublicKey, Signature};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::Token;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, Gas, Promise, PromiseError};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
pub mod events;
pub mod external;
pub use crate::events::*;
pub use crate::external::*;

const GAS_FOR_FREEZE_NFT: Gas = Gas(35_000_000_000_000);
const GAS_FOR_WITHDRAW_NFT: Gas = Gas(30_000_000_000_000);
const GAS_FOR_VALIDATE_TRANSFER: Gas = Gas(30_000_000_000_000);
const GAS_FOR_VALIDATE_UNFREEZE: Gas = Gas(35_000_000_000_000);

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
    pub account_id: AccountId,
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
    /// Initializes the contract with the provided group key.
    /// Also sets the initial action count, whitelist, and
    /// other contract state variables.
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
    fn require_sig(&mut self, action_id: u128, data: Vec<u8>, sig_data: Vec<u8>, context: &[u8]) {
        let f = self.consumed_actions.contains_key(&action_id);
        require!(!f, "Duplicated Action");

        self.consumed_actions.insert(action_id, true);

        let mut hasher = Sha512::new();
        hasher.update(context);
        hasher.update(data);
        let hash = hasher.finalize();

        let sig = Signature::new(sig_data.as_slice().try_into().unwrap());
        let key = PublicKey::new(self.group_key);
        let res = key.verify(hash, &sig);
        require!(res.is_ok(), "Unauthorized Action");
    }

    /// Pauses the contract which will stop all bridge actions from being executed.
    /// /// FAILS: If already paused.
    /// REQUIRED: Signature verification.
    pub fn validate_pause(&mut self, data: PauseData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"SetPause",
        );

        self.paused = true;
    }

    /// Unpauses the contract which will stop all bridge actions from being executed.
    /// FAILS: If already unpaused.
    /// REQUIRED: Signature verification.
    pub fn validate_unpause(&mut self, data: UnpauseData, sig_data: Vec<u8>) {
        require!(self.paused, "unpaused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"SetUnpause",
        );

        self.paused = false;
    }

    /// Withdraws the fees collected by the contract on NFT transfers.
    /// to the account_id provied in the {WithdrawFeeData}.
    /// FAILS: If contract is paused.
    /// REQUIRED: Signature verification.
    pub fn validate_withdraw_fees(&mut self, data: WithdrawFeeData, sig_data: Vec<u8>) -> Promise {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"WithdrawFees",
        );

        let storage_used = env::storage_usage();
        let amt = env::account_balance() - storage_used as u128 * env::storage_byte_cost();
        Promise::new(data.account_id)
            .transfer(amt)
            .then(Self::ext(env::current_account_id()).withdraw_fee_callback())
    }

    /// This is the callback function when the promise in the
    /// validate_withdraw_fees function is completed. It will
    /// check if the promise result was successful or not.
    #[private]
    pub fn withdraw_fee_callback(
        &mut self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        require!(call_result.is_ok(), "withdraw failed");

        self.tx_fees = 0;
    }
    /// Updates the Group Key for the contract.
    /// FAILS: If contract is paused.
    /// REQUIRED: Signature verification.
    pub fn validate_update_group_key(&mut self, data: UpdateGroupkeyData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"SetGroupKey",
        );

        self.group_key = data.group_key;
    }

    /// Updates the whitelist for the contract.
    /// Adds the provided account_id to the whitelist
    /// so that they can be freezed for transfers to work
    /// in the bridge
    /// FAILS: If contract is paused.
    /// REQUIRED: Signature verification.
    pub fn validate_whitelist(&mut self, data: WhitelistData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        require!(
            !self
                .whitelist
                .contains_key(&data.token_contract.to_string()),
            "Already whitelist"
        );

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"WhitelistNft",
        );

        self.whitelist.insert(data.token_contract, true);
    }
    /// Updates the whitelist for the contract.
    /// Removes the provided account_id from the whitelist
    /// so that they cannot be freezed for transfers to work
    /// in the bridge
    /// FAILS: If contract is paused AND if the contract is not present in whitelist.
    /// REQUIRED: Signature verification.
    pub fn validate_blacklist(&mut self, data: WhitelistData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        require!(
            self.whitelist
                .contains_key(&data.token_contract.to_string()),
            "Not whitelist"
        );

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"ValidateBlacklistNft",
        );

        self.whitelist.remove(&data.token_contract);
    }

    /// Validates the transfer of NFT from the bridge to the destination chain.
    /// It mints a new NEP-171 token on chain to the destination account_id.
    /// FAILS: If contract is paused.
    /// REQUIRED: Signature verification.
    #[payable]
    pub fn validate_transfer_nft(&mut self, data: TransferNftData, sig_data: Vec<u8>) -> Promise {
        require!(
            env::prepaid_gas() > GAS_FOR_VALIDATE_TRANSFER,
            "Not enough gas"
        );
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data.into(),
            b"ValidateTransferNft",
        );

        xpnft::ext(data.mint_with)
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(TGAS * 10))
            .nft_mint(data.token_id, data.owner_id, data.token_metadata)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 10))
                    .validate_transfer_callback(data.action_id.0),
            )
    }

    // This is the callback function when the promise in the validate_unfreeze_nft
    /// function is completed. It will check if the promise result was
    /// successful or not.
    #[private]
    pub fn validate_transfer_callback(
        &mut self,
        action_id: u128,
        #[callback_result] call_result: Result<Token, PromiseError>,
    ) {
      let _res = match call_result {
            Ok(_) => {
                // Do Nothing
            }
            Err(_e) => {
                self.consumed_actions.remove(&action_id);
            },
        };
    }

    /// Withdraw foreign NFT. This creates a promise to get the token data
    /// from the foreign contract and then calls the callback function
    /// 'token_callback'.
    /// WARN: Even though this contract doesn't check if the burner is trusted,
    /// we check this in the bridge infrastructure(i.e in the validator)
    #[payable]
    pub fn withdraw_nft(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String
    ) -> Promise {
        require!(env::prepaid_gas() > GAS_FOR_WITHDRAW_NFT, "Not enough gas");
        require!(!self.paused, "paused");

        require!(env::attached_deposit() > 0, "the attached deposit must not be zero and should be equal to the parameter amt of this function");

        xpnft::ext(token_contract.clone())
            .with_static_gas(Gas(TGAS * 10))
            .nft_token(token_id.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 25))
                    .token_callback(
                        token_contract,
                        token_id,
                        env::predecessor_account_id(),
                        chain_nonce,
                        to,
                        env::attached_deposit()
                    ),
            )
    }

    /// This is the callback function when the promise in the withdraw_nft
    /// function is completed. It will check if the promise result was
    /// successful or not. If it was successful, it will create a nft burn
    /// promise and then call the callback function 'burn_callback'.
    #[private]
    pub fn token_callback(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        owner_id: AccountId,
        chain_nonce: u8,
        to: String,
        amt: u128,
        #[callback_result] call_result: Result<Option<Token>, PromiseError>,
    ) -> Promise {
        require!(call_result.is_ok(), "token callback failed");

        xpnft::ext(token_contract.clone())
            .with_static_gas(Gas(TGAS * 10))
            .nft_burn(token_id.clone(), owner_id)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 10))
                    .withdraw_callback(
                        token_contract,
                        call_result.unwrap(),
                        chain_nonce,
                        to,
                        amt.into(),
                    ),
            )
    }

    /// This is the callback function when the promise in the token_callback
    /// function is completed. It will check if the promise result was
    /// successful or not. If it was successful, it will emit an unfreeze nft event
    #[private]
    pub fn withdraw_callback(
        &mut self,
        token_contract: AccountId,
        token: Option<Token>,
        chain_nonce: u8,
        to: String,
        amt: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        require!(call_result.is_ok(), "withdraw failed");

        self.action_cnt += 1;
        self.tx_fees += amt;

        UnfreezeNftEvent {
            action_id: self.action_cnt,
            chain_nonce,
            to,
            amt,
            contract: token_contract,
            token,
        }
        .emit();
    }

    /// Freezes the NFT on the bridge contract. NFT is transferred to this
    /// bridge contract with a promise and then on completion of the promise
    /// the callback function `freeze_callback` is called.
    #[payable]
    pub fn freeze_nft(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        mint_with: String,
    ) -> Promise {
        require!(env::prepaid_gas() > GAS_FOR_FREEZE_NFT, "Not enough gas");
        require!(!self.paused, "paused");

        require!(
            self.whitelist
                .contains_key(&token_contract.clone().to_string()),
            "Not whitelist"
        );

        require!(env::attached_deposit() > 0, "the attached deposit must not be zero and should be equal to the parameter amt of this function");

        common_nft::ext(token_contract.clone())
            .with_attached_deposit(1)
            .with_static_gas(Gas(TGAS * 15))
            .nft_transfer(env::current_account_id(), token_id.clone(), None, None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 10))
                    .freeze_callback(
                        token_contract,
                        token_id,
                        chain_nonce,
                        to,
                        mint_with,
                        env::attached_deposit()
                    ),
            )
    }

    /// This is the callback function when the promise in the freeze_nft
    /// function is completed. It will check if the promise result was
    /// successful or not. If it was successful, it will emit a TransferNftEvent
    /// event.
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
    ) {
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
    }

    /// This function unfreezes the NFT on the bridge contract.
    /// It will transfer the NFT from this contract to the receiver
    /// contract.
    pub fn validate_unfreeze_nft(&mut self, data: UnfreezeNftData, sig_data: Vec<u8>) -> Promise {
        require!(
            env::prepaid_gas() > GAS_FOR_VALIDATE_UNFREEZE,
            "Not enough gas"
        );
        require!(!self.paused, "paused");

        require!(
            self.whitelist
                .contains_key(&data.token_contract.clone().to_string()),
            "Not whitelist"
        );

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"ValidateUnfreezeNft",
        );

        common_nft::ext(data.token_contract)
            .with_static_gas(Gas(TGAS * 20))
            .nft_transfer(data.receiver_id, data.token_id, None, None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 8))
                    .validate_unfreeze_callback(data.action_id.0),
            )
    }

    /// This is the callback function when the promise in the validate_unfreeze_nft
    /// function is completed. It will check if the promise result was
    /// successful or not.
    #[private]
    pub fn validate_unfreeze_callback(
        &mut self,
        action_id: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        let _res = match call_result {
            Ok(_) => {
                // Do Nothing
            }
            Err(_e) => {
                self.consumed_actions.remove(&action_id);
            },
        };
    }

    /// This function takes all the parameters of the TransferNftData
    /// and then encodes into Bytes (Vec<u8>) which is consumed by the
    /// validator for signing the transaction.
    pub fn encode_transfer_action(
        &self,
        action_id: U128,
        mint_with: AccountId,
        owner_id: AccountId,
        token_id: String,
        title: String,
        description: String,
        media: String,
        extra: String,
    ) -> Vec<u8> {
        let data = TransferNftData {
            action_id,
            mint_with,
            owner_id,
            token_id,
            token_metadata: TokenMetadata {
                title: Some(title),
                description: Some(description),
                media: Some(media),
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: Some(extra),
                reference: None,
                reference_hash: None,
            },
        };
        data.try_to_vec().unwrap()
    }

    /// This function takes all the parameters of the UnfreezeNftData
    /// and then encodes into Bytes (Vec<u8>) which is consumed by the
    /// validator for signing the transaction.
    pub fn encode_unfreeze_action(
        &self,
        action_id: U128,
        token_id: String,
        receiver_id: AccountId,
        token_contract: AccountId,
    ) -> Vec<u8> {
        let event = UnfreezeNftData {
            action_id,
            token_id,
            receiver_id,
            token_contract,
        };
        event.try_to_vec().unwrap()
    }

    /// Gets the currently set group key from the state variables of the contract
    pub fn get_group_key(&self) -> [u8; 32] {
        self.group_key
    }

    /// Checks if the contract provided in `contract_id` is whitelisted
    /// or not.
    /// Returns boolean
    pub fn is_whitelist(&self, contract_id: String) -> bool {
        self.whitelist.contains_key(&contract_id)
    }

    /// Checks if the contract is paused or not.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Gets the no of actions performed by the contract.
    pub fn get_action_cnt(&self) -> U128 {
        U128(self.action_cnt)
    }
}
