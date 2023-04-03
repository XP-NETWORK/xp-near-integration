use ed25519_compact::{PublicKey, Signature};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
// use near_contract_standards::non_fungible_token::Token;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::PanicOnDefault;
use near_sdk::ONE_NEAR;
use near_sdk::{env, near_bindgen, require, AccountId, Gas, Promise, PromiseError};
use sha2::{Digest, Sha512};
pub mod events;
pub mod external;
pub use crate::events::*;
pub use crate::external::*;
use std::collections::HashMap;

const GAS_FOR_FREEZE_NFT: Gas = Gas(45_000_000_000_000);
const GAS_FOR_WITHDRAW_NFT: Gas = Gas(65_000_000_000_000);
const GAS_FOR_VALIDATE_TRANSFER: Gas = Gas(35_000_000_000_000);
const GAS_FOR_VALIDATE_WITHDRAW: Gas = Gas(35_000_000_000_000);
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

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferTx {
    value: u128,
    from_chain: u8,
    to_chain: u8,
    to: String,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferNftData {
    action_id: U128,
    mint_with: AccountId,
    token_id: TokenId,
    owner_id: AccountId,
    token_metadata: TokenMetadata,
    perpetual_royalties: Option<HashMap<AccountId, u32>>,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferMtData {
    action_id: U128,
    mint_with: AccountId,
    owner_id: AccountId,
    token_metadatas: Vec<TokenMetadata>,
    supply: Vec<u128>,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnfreezeNftData {
    action_id: U128,
    token_contract: AccountId,
    token_id: TokenId,
    receiver_id: AccountId,
    // token_amt: U128,
}
#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnfreezeMtData {
    action_id: u128,
    token_contract: AccountId,
    token_ids: Vec<TokenId>,
    receiver_id: AccountId,
    token_amts: Vec<u128>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct XpBridge {
    consumed_actions: UnorderedSet<u128>,
    paused: bool,
    tx_fees: u128,
    group_key: [u8; 32],
    fee_pk: [u8; 32],
    action_cnt: u128,
}

#[near_bindgen]
impl XpBridge {
    /// Initializes the contract with the provided group key.
    /// Also sets the initial action count, and
    /// other contract state variables.
    #[init]
    pub fn initialize(group_key: [u8; 32], fee_pk: [u8; 32]) -> Self {
        assert!(
            env::current_account_id() == env::predecessor_account_id(),
            "Unauthorized"
        );

        Self {
            consumed_actions: UnorderedSet::new(b"c"),
            paused: false,
            fee_pk,
            tx_fees: 0,
            group_key,
            action_cnt: 0,
        }
    }

    /// Ed25519 Signature verification logic.
    /// Signature check for bridge actions.
    /// Consumes the passed action_id.
    fn require_sig(&mut self, action_id: u128, data: Vec<u8>, sig_data: Vec<u8>, context: &[u8]) {
        let f = self.consumed_actions.contains(&action_id);
        require!(!f, "Duplicated Action");

        self.consumed_actions.insert(&action_id);

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
        require!(
            env::prepaid_gas() >= GAS_FOR_VALIDATE_WITHDRAW,
            "Not enough gas"
        );

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"WithdrawFees",
        );

        let storage_used = env::storage_usage();
        let amt =
            (env::account_balance() - storage_used as u128 * env::storage_byte_cost()) - ONE_NEAR;
        Promise::new(data.account_id).transfer(amt).then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas(TGAS * 15))
                .withdraw_fee_callback(data.action_id.0),
        )
    }

    /// This is the callback function when the promise in the
    /// validate_withdraw_fees function is completed. It will
    /// check if the promise result was successful or not.
    #[private]
    pub fn withdraw_fee_callback(
        &mut self,
        action_id: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Err(e) => {
                self.consumed_actions.remove(&action_id);
                env::log_str(&format!(
                    "validate transfer callback: failed to transfer tokens: actionid: {} : {:?}",
                    action_id, e
                ))
            }
            Ok(_) => {
                // Do nothing
                self.tx_fees = 0;
            }
        }
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

    pub fn validate_update_fee_public_key(&mut self, data: UpdateGroupkeyData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"SetFeePublicKey",
        );

        self.fee_pk = data.group_key;
    }
    /// so that they cannot be freezed for transfers to work
    /// in the bridge
    /// FAILS: If contract is paused AND if the contract is not present in whitelist.
    /// REQUIRED: Signature verification.
    pub fn validate_blacklist(&mut self, data: WhitelistData, sig_data: Vec<u8>) {
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"ValidateBlacklistNft",
        );
    }

    /// Validates the transfer of NFT from the bridge to the destination chain.
    /// It mints a new NEP-171 token on chain to the destination account_id.
    /// FAILS: If contract is paused.
    /// REQUIRED: Signature verification.
    #[payable]
    pub fn validate_transfer_nft(&mut self, data: TransferNftData, sig_data: Vec<u8>) -> Promise {
        require!(
            env::prepaid_gas() >= GAS_FOR_VALIDATE_TRANSFER,
            "Not enough gas"
        );
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"ValidateTransferNft",
        );

        xpnft_royalty::ext(data.mint_with)
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(TGAS * 10))
            .nft_mint(
                data.token_id,
                data.owner_id,
                data.token_metadata,
                data.perpetual_royalties,
            )
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
            Err(e) => {
                self.consumed_actions.remove(&action_id);
                env::log_str(&format!(
                    "validate transfer callback: failed to mint nft: actionid: {} : {:?}",
                    action_id, e
                ))
            }
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
        to: String,
        sig_data: Vec<u8>,
    ) -> Promise {
        require!(env::prepaid_gas() >= GAS_FOR_WITHDRAW_NFT, "Not enough gas");
        require!(!self.paused, "paused");

        return Self::ext(env::current_account_id())
            .verify_paid_amount_by_sig(
                TransferTx {
                    value: env::attached_deposit().into(),
                    from_chain: 31,
                    to_chain: chain_nonce,
                    to: to.clone(),
                },
                sig_data,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 45))
                    .check_enough_fees_callback_for_withdraw(
                        token_contract,
                        token_id,
                        chain_nonce,
                        to,
                        env::attached_deposit(),
                        env::signer_account_id(),
                    ),
            );
    }

    #[private]
    pub fn check_enough_fees_callback_for_withdraw(
        &self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        amt: u128,
        sender: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
                xpnft_royalty::ext(token_contract.clone())
                    .with_static_gas(Gas(5 * TGAS))
                    .nft_token(token_id.clone())
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(TGAS * 25))
                            .token_callback(
                                token_contract,
                                token_id,
                                sender,
                                chain_nonce,
                                to,
                                env::attached_deposit(),
                            ),
                    );
            }
            Err(e) => {
                Promise::new(sender).transfer(amt);
                env::log_str(&format!(
                    "withdraw callback: failed to transfer nft: failed to get tx fee :actionid: {} : {:?}",
                    self.action_cnt, e
                ))
            }
        }
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
        #[callback_result] call_result: Result<Option<JsonToken>, PromiseError>,
    ) -> Promise {
        match call_result {
            Ok(_) => xpnft_royalty::ext(token_contract.clone())
                .with_static_gas(Gas(TGAS * 10))
                .nft_burn(token_id.clone(), owner_id)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas(TGAS * 8))
                        .withdraw_callback(
                            token_contract,
                            call_result.unwrap(),
                            chain_nonce,
                            to,
                            amt.into(),
                            env::predecessor_account_id(),
                        ),
                ),
            Err(_) => {
                // Return funds
                Promise::new(env::signer_account_id()).transfer(amt)
            }
        }
    }

    /// This is the callback function when the promise in the token_callback
    /// function is completed. It will check if the promise result was
    /// successful or not. If it was successful, it will emit an unfreeze nft event
    #[private]
    pub fn withdraw_callback(
        &mut self,
        token_contract: AccountId,
        token: Option<JsonToken>,
        chain_nonce: u8,
        to: String,
        amt: u128,
        sender: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
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
            Err(e) => {
                Promise::new(sender).transfer(amt);
                env::log_str(&format!(
                    "validate withdraw callback: failed to burn nft:  actionid: {} : {:?}",
                    self.action_cnt, e
                ))
            }
        }
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
        sig_data: Vec<u8>,
        // token_amt: Option<u128>,
    ) -> Promise {
        require!(env::prepaid_gas() >= GAS_FOR_FREEZE_NFT, "Not enough gas");
        require!(!self.paused, "paused");

        return Self::ext(env::current_account_id())
            .verify_paid_amount_by_sig(
                TransferTx {
                    value: env::attached_deposit().into(),
                    from_chain: 31,
                    to_chain: chain_nonce,
                    to: to.clone(),
                },
                sig_data,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 30))
                    .check_enough_fees_callback_for_transfer(
                        token_contract,
                        token_id,
                        chain_nonce,
                        to,
                        mint_with,
                        env::attached_deposit(),
                        env::signer_account_id(),
                        // token_amt,
                    ),
            );
    }

    #[private]
    pub fn check_enough_fees_callback_for_transfer(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        mint_with: String,
        amt: u128,
        sender: AccountId,
        // token_amt: Option<u128>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
                // if token_amt > Some(0) {
                //     common_mt::ext(token_contract.clone())
                //         .with_attached_deposit(1)
                //         .with_static_gas(Gas(TGAS * 10))
                //         .mt_transfer(
                //             env::current_account_id(),
                //             token_id.clone(),
                //             match token_amt {
                //                 Some(p) => p,
                //                 None => 0,
                //             },
                //             None,
                //             None,
                //         )
                //         .then(
                //             Self::ext(env::current_account_id())
                //                 .with_static_gas(Gas(TGAS * 8))
                //                 .freeze_callback(
                //                     token_contract,
                //                     token_id,
                //                     chain_nonce,
                //                     to,
                //                     mint_with,
                //                     amt,
                //                     sender,
                //                     token_amt,
                //                 ),
                //         );
                // } else {
                common_nft::ext(token_contract.clone())
                    .with_attached_deposit(1)
                    .with_static_gas(Gas(TGAS * 10))
                    .nft_transfer(env::current_account_id(), token_id.clone(), None, None)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(TGAS * 8))
                            .freeze_callback(
                                token_contract,
                                token_id,
                                chain_nonce,
                                to,
                                mint_with,
                                amt,
                                sender,
                                // token_amt,
                            ),
                    );
                // }
            }
            Err(e) => {
                Promise::new(sender).transfer(amt);
                let msg;
                // if token_amt > Some(0) {
                //     msg = format!("freeze callback: failed to transfer mt: failed to verify tx fee :actionid: {} : {:?}",
                //     self.action_cnt, e);
                // } else {
                msg = format!("freeze callback: failed to transfer nft: failed to verify tx fee :actionid: {} : {:?}",
                    self.action_cnt, e);
                // }
                env::log_str(&msg)
            }
        }
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
        sender: AccountId,
        // token_amt: Option<u128>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
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
                    // token_amt,
                }
                .emit();
            }
            Err(e) => {
                Promise::new(sender).transfer(amt);
                let msg;
                // if token_amt > Some(0) {
                //     msg = format!(
                //         "freeze callback: failed to transfer mt: actionid: {} : {:?}",
                //         self.action_cnt, e
                //     );
                // } else {
                msg = format!(
                    "freeze callback: failed to transfer nft: actionid: {} : {:?}",
                    self.action_cnt, e
                );
                // }
                env::log_str(&msg);
            }
        }
    }

    /// This function unfreezes the NFT on the bridge contract.
    /// It will transfer the NFT from this contract to the receiver
    /// contract.
    #[payable]
    pub fn validate_unfreeze_nft(&mut self, data: UnfreezeNftData, sig_data: Vec<u8>) -> Promise {
        require!(
            env::prepaid_gas() >= GAS_FOR_VALIDATE_UNFREEZE,
            "Not enough gas"
        );
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"ValidateUnfreezeNft",
        );

        // if data.token_amt > U128(0) {
        //     common_mt::ext(data.token_contract)
        //         .with_attached_deposit(env::attached_deposit())
        //         .with_static_gas(Gas(TGAS * 10))
        //         .mt_transfer(
        //             data.receiver_id,
        //             data.token_id,
        //             data.token_amt.into(),
        //             None,
        //             None,
        //         )
        //         .then(
        //             Self::ext(env::current_account_id())
        //                 .with_static_gas(Gas(TGAS * 10))
        //                 .validate_unfreeze_callback(data.action_id.0, data.token_amt.0),
        //         )
        // } else {
        common_nft::ext(data.token_contract)
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(TGAS * 10))
            .nft_transfer(data.receiver_id, data.token_id, None, None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 10))
                    .validate_unfreeze_callback(data.action_id.0 /* , data.token_amt.0*/),
            )
        // }
    }

    /// This is the callback function when the promise in the validate_unfreeze_nft
    /// function is completed. It will check if the promise result was
    /// successful or not.
    #[private]
    pub fn validate_unfreeze_callback(
        &mut self,
        action_id: u128,
        // token_amt: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        let _res = match call_result {
            Ok(_) => {
                // Do Nothing
            }
            Err(e) => {
                self.consumed_actions.remove(&action_id);
                let msg;
                // if token_amt > U128(0).into() {
                //     msg = format!(
                //         "validate unfreeze callback: failed to transfer mt: action id: {}: {:?}",
                //         action_id, e
                //     );
                // } else {
                msg = format!(
                    "validate unfreeze callback: failed to transfer nft: action id: {}: {:?}",
                    action_id, e
                );
                // }
                env::log_str(&msg);
            }
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
        reference: String,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
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
                reference: Some(reference),
                reference_hash: None,
            },
            perpetual_royalties,
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
        // token_amt: U128,
    ) -> Vec<u8> {
        let event = UnfreezeNftData {
            action_id,
            token_id,
            receiver_id,
            token_contract,
            // token_amt,
        };
        event.try_to_vec().unwrap()
    }

    pub fn encode_whitelist_action(&self, action_id: U128, token_contract: String) -> Vec<u8> {
        let event = WhitelistData {
            action_id,
            token_contract,
        };
        event.try_to_vec().unwrap()
    }

    /// This function takes all the parameters of the TransferNftData
    /// and then encodes into Bytes (Vec<u8>) which is consumed by the
    /// validator for signing the transaction.
    pub fn encode_transfer_action_mt(
        &self,
        action_id: U128,
        mint_with: AccountId,
        owner_id: AccountId,
        token_metadatas: Vec<TokenMetadata>,
        supply: Vec<u128>,
    ) -> Vec<u8> {
        let data = TransferMtData {
            action_id,
            mint_with,
            owner_id,
            token_metadatas,
            supply,
        };
        data.try_to_vec().unwrap()
    }

    /// This function takes all the parameters of the UnfreezeNftData
    /// and then encodes into Bytes (Vec<u8>) which is consumed by the
    /// validator for signing the transaction.
    pub fn encode_unfreeze_action_mt(
        &self,
        action_id: U128,
        token_ids: Vec<TokenId>,
        receiver_id: AccountId,
        token_contract: AccountId,
        token_amts: Vec<u128>,
    ) -> Vec<u8> {
        let event = UnfreezeMtData {
            action_id: action_id.into(),
            token_contract,
            token_ids,
            receiver_id,
            token_amts: token_amts.into(),
        };
        event.try_to_vec().unwrap()
    }

    /// Gets the currently set group key from the state variables of the contract
    pub fn get_group_key(&self) -> [u8; 32] {
        self.group_key
    }

    pub fn get_fee_key(&self) -> [u8; 32] {
        self.fee_pk
    }

    /// Checks if the contract is paused or not.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Gets the no of actions performed by the contract.
    pub fn get_action_cnt(&self) -> U128 {
        U128(self.action_cnt)
    }

    #[payable]
    pub fn freeze_mt(
        &mut self,
        token_contract: AccountId,
        token_id: TokenId,
        chain_nonce: u8,
        to: String,
        mint_with: String,
        sig_data: Vec<u8>,
    ) -> Promise {
        require!(env::prepaid_gas() >= GAS_FOR_FREEZE_NFT, "Not enough gas");
        require!(!self.paused, "paused");
        return Self::ext(env::current_account_id())
            .verify_paid_amount_by_sig(
                TransferTx {
                    value: env::attached_deposit().into(),
                    from_chain: 31,
                    to_chain: chain_nonce,
                    to: to.clone(),
                },
                sig_data,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 30))
                    .check_enough_fees_callback_for_transfer_mt(
                        token_contract,
                        vec![token_id],
                        chain_nonce,
                        to,
                        mint_with,
                        env::attached_deposit(),
                        env::signer_account_id(),
                        vec![1],
                    ),
            );
    }

    #[payable]
    pub fn freeze_mt_batch(
        &mut self,
        token_contract: AccountId,
        token_ids: Vec<TokenId>,
        chain_nonce: u8,
        to: String,
        mint_with: String,
        sig_data: Vec<u8>,
        token_amts: Vec<u128>,
    ) -> Promise {
        require!(env::prepaid_gas() >= GAS_FOR_FREEZE_NFT, "Not enough gas");
        require!(!self.paused, "paused");

        return Self::ext(env::current_account_id())
            .verify_paid_amount_by_sig(
                TransferTx {
                    value: env::attached_deposit().into(),
                    from_chain: 31,
                    to_chain: chain_nonce,
                    to: to.clone(),
                },
                sig_data,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 30))
                    .check_enough_fees_callback_for_transfer_mt(
                        token_contract,
                        token_ids,
                        chain_nonce,
                        to,
                        mint_with,
                        env::attached_deposit(),
                        env::signer_account_id(),
                        token_amts,
                    ),
            );
    }

    #[private]
    pub fn check_enough_fees_callback_for_transfer_mt(
        &mut self,
        token_contract: AccountId,
        token_ids: Vec<TokenId>,
        chain_nonce: u8,
        to: String,
        mint_with: String,
        amt: u128,
        sender: AccountId,
        token_amts: Vec<u128>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
                common_mt::ext(token_contract.clone())
                    .with_attached_deposit(1)
                    .with_static_gas(Gas(TGAS * 10))
                    .mt_batch_transfer(
                        env::current_account_id(),
                        token_ids.clone(),
                        token_amts.clone(),
                        None,
                        None,
                    )
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(TGAS * 8))
                            .freeze_callback_mt(
                                token_contract,
                                token_ids,
                                chain_nonce,
                                to,
                                mint_with,
                                amt,
                                sender,
                                token_amts,
                            ),
                    );
            }
            Err(e) => {
                Promise::new(sender).transfer(amt);
                let msg;
                msg = format!("freeze callback: failed to transfer mt: failed to verify tx fee :actionid: {} : {:?}",
                    self.action_cnt, e);
                env::log_str(&msg)
            }
        }
    }

    #[private]
    pub fn freeze_callback_mt(
        &mut self,
        token_contract: AccountId,
        token_id: Vec<TokenId>,
        chain_nonce: u8,
        to: String,
        mint_with: String,
        amt: u128,
        sender: AccountId,
        token_amts: Vec<u128>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
                self.action_cnt += 1;
                self.tx_fees += amt;

                TransferMtEvent {
                    action_id: self.action_cnt,
                    chain_nonce,
                    to,
                    amt,
                    contract: token_contract,
                    token_id,
                    mint_with,
                    token_amts,
                }
                .emit();
            }
            Err(e) => {
                Promise::new(sender).transfer(amt);
                let msg;
                msg = format!(
                    "freeze callback: failed to transfer nft: actionid: {} : {:?}",
                    self.action_cnt, e
                );
                env::log_str(&msg);
            }
        }
    }

    #[payable]
    pub fn withdraw_mt(
        &mut self,
        token_contract: AccountId,
        token_ids: Vec<TokenId>,
        chain_nonce: u8,
        to: String,
        sig_data: Vec<u8>,
        token_amts: Vec<u128>,
    ) -> Promise {
        require!(env::prepaid_gas() >= GAS_FOR_WITHDRAW_NFT, "Not enough gas");
        require!(!self.paused, "paused");

        return Self::ext(env::current_account_id())
            .verify_paid_amount_by_sig(
                TransferTx {
                    value: env::attached_deposit().into(),
                    from_chain: 31,
                    to_chain: chain_nonce,
                    to: to.clone(),
                },
                sig_data,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 45))
                    .check_enough_fees_callback_for_withdraw_mt(
                        token_contract,
                        token_ids,
                        chain_nonce,
                        to,
                        env::attached_deposit(),
                        env::signer_account_id(),
                        token_amts,
                    ),
            );
    }

    #[private]
    pub fn check_enough_fees_callback_for_withdraw_mt(
        &self,
        token_contract: AccountId,
        token_ids: Vec<TokenId>,
        chain_nonce: u8,
        to: String,
        amt: u128,
        sender: AccountId,
        token_amts: Vec<u128>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
                xpnft_erc1155::ext(token_contract.clone())
                    .with_static_gas(Gas(5 * TGAS))
                    .mt_token(token_ids.clone())
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(TGAS * 25))
                            .multitoken_callback(
                                token_contract,
                                token_ids,
                                sender,
                                chain_nonce,
                                to,
                                env::attached_deposit(),
                                token_amts,
                            ),
                    );
            }
            Err(e) => {
                Promise::new(sender).transfer(amt);
                env::log_str(&format!(
                    "withdraw callback: failed to transfer mt: failed to get tx fee :actionid: {} : {:?}",
                    self.action_cnt, e
                ))
            }
        }
    }

    #[private]
    pub fn multitoken_callback(
        &mut self,
        token_contract: AccountId,
        token_ids: Vec<TokenId>,
        owner_id: AccountId,
        chain_nonce: u8,
        to: String,
        amt: u128,
        token_amts: Vec<u128>,
        #[callback_result] call_result: Result<Option<MultiToken>, PromiseError>,
    ) -> Promise {
        match call_result {
            Ok(_) => xpnft_erc1155::ext(token_contract.clone())
                .with_static_gas(Gas(TGAS * 10))
                .mt_burn(token_ids.clone(), token_amts.clone(), owner_id)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas(TGAS * 8))
                        .withdraw_callback_mt(
                            token_contract,
                            call_result.unwrap(),
                            chain_nonce,
                            to,
                            amt.into(),
                            env::predecessor_account_id(),
                            token_amts,
                        ),
                ),
            Err(_) => {
                // Return funds
                Promise::new(env::signer_account_id()).transfer(amt)
            }
        }
    }

    #[private]
    pub fn withdraw_callback_mt(
        &mut self,
        token_contract: AccountId,
        tokens: Option<MultiToken>,
        chain_nonce: u8,
        to: String,
        amt: u128,
        sender: AccountId,
        token_amts: Vec<u128>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        match call_result {
            Ok(_) => {
                self.action_cnt += 1;
                self.tx_fees += amt;

                UnfreezeMtEvent {
                    action_id: self.action_cnt,
                    chain_nonce,
                    to,
                    amt,
                    contract: token_contract,
                    tokens,
                    token_amts,
                }
                .emit();
            }
            Err(e) => {
                Promise::new(sender).transfer(amt);
                env::log_str(&format!(
                    "validate withdraw callback: failed to burn mt:  actionid: {} : {:?}",
                    self.action_cnt, e
                ))
            }
        }
    }

    #[payable]
    pub fn validate_unfreeze_mt_batch(
        &mut self,
        data: UnfreezeMtData,
        sig_data: Vec<u8>,
    ) -> Promise {
        require!(
            env::prepaid_gas() >= GAS_FOR_VALIDATE_UNFREEZE,
            "Not enough gas"
        );
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"ValidateUnfreezeMtBatch",
        );

        common_mt::ext(data.token_contract)
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(TGAS * 10))
            .mt_batch_transfer(
                data.receiver_id,
                data.token_ids,
                data.token_amts,
                None,
                None,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 10))
                    .validate_unfreeze_callback_batch(data.action_id),
            )
    }

    /// This is the callback function when the promise in the validate_unfreeze_nft
    /// function is completed. It will check if the promise result was
    /// successful or not.
    #[private]
    pub fn validate_unfreeze_callback_batch(
        &mut self,
        action_id: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        let _res = match call_result {
            Ok(_) => {
                // Do Nothing
            }
            Err(e) => {
                self.consumed_actions.remove(&action_id);
                let msg = format!(
                    "validate unfreeze callback: failed to transfer mt: action id: {}: {:?}",
                    action_id, e
                );
                env::log_str(&msg);
            }
        };
    }

    #[payable]
    pub fn validate_transfer_mt(&mut self, data: TransferMtData, sig_data: Vec<u8>) -> Promise {
        require!(
            env::prepaid_gas() >= GAS_FOR_VALIDATE_TRANSFER,
            "Not enough gas"
        );
        require!(!self.paused, "paused");

        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"ValidateTransferMt",
        );

        xpnft_erc1155::ext(data.mint_with)
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(TGAS * 10))
            .mt_mint(data.owner_id, data.token_metadatas, data.supply)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(TGAS * 10))
                    .validate_transfer_callback_mt(data.action_id.0),
            )
    }

    // This is the callback function when the promise in the validate_unfreeze_nft
    /// function is completed. It will check if the promise result was
    /// successful or not.
    #[private]
    pub fn validate_transfer_callback_mt(
        &mut self,
        action_id: u128,
        #[callback_result] call_result: Result<Vec<MultiToken>, PromiseError>,
    ) {
        let _res = match call_result {
            Ok(_) => {
                // Do Nothing
            }
            Err(e) => {
                self.consumed_actions.remove(&action_id);
                env::log_str(&format!(
                    "validate transfer callback: failed to mint mt: actionid: {} : {:?}",
                    action_id, e
                ))
            }
        };
    }

    #[private]
    pub fn verify_paid_amount_by_sig(&self, data: TransferTx, sig_data: Vec<u8>) {
        let mut hasher = Sha512::new();
        hasher.update(data.try_to_vec().unwrap());
        let hash = hasher.finalize();
        let sig = Signature::new(sig_data.as_slice().try_into().unwrap());
        let key = PublicKey::new(self.fee_pk);
        let _ = key
            .verify(hash, &sig)
            .expect("Amount Signature Verification Failed");
    }
}
