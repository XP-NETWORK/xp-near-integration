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
    mint_with: String,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawFeeData {
    pub action_id: u128,
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

    // #[payable]
    // pub fn validate_withdraw_fees(
    //     &mut self,
    //     data: WithdrawFeeData,
    //     sig_data: Vec<u8>,
    // ) -> PromiseOrValue<()> {
    //     require!(!self.paused, "paused");

    //     self.require_sig(
    //         data.action_id,
    //         data.try_to_vec().unwrap(),
    //         sig_data,
    //         b"WithdrawFees",
    //     );

    //     let account_id: AccountId = data.account_id.parse().unwrap();
    //     let public_key = near_sdk::PublicKey::try_from(data.public_key).unwrap();
    //     // Creating new account. It still can fail (e.g. account already exists or name is invalid),
    //     // but we don't care, we'll get a refund back.
    //     Promise::new(account_id)
    //         .create_account()
    //         .transfer(env::account_balance() - 10_000_000)
    //         .add_full_access_key(public_key)
    //         .into()
    // }

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

        self.require_sig(
            data.action_id.into(),
            sha256(data.try_to_vec().unwrap().as_slice()),
            sig_data.into(),
        );

        self.whitelist.insert(data.mint_with, true);
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
            .with_attached_deposit(6_150_000_000 * TYOCTO)
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
            .then(Promise::new(env::current_account_id()).transfer(amt.into()))
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
            token_id,
        }
        .emit();
    }

    // /// Freeze NEP-171 token.
    // #[payable]
    // pub fn freeze_nft(
    //     &mut self,
    //     token_id: TokenId,
    //     chain_nonce: u8,
    //     to: String,
    //     amt: u128,
    //     mint_with: String,
    //     token_contract: AccountId,
    // ) {
    //     require!(!self.paused, "paused");

    //     common_nft::ext(token_contract.clone()).nft_transfer(
    //         env::current_account_id(),
    //         token_id.clone(),
    //         None,
    //         None,
    //     );

    //     Promise::new(env::current_account_id()).transfer(amt);

    //     self.action_cnt += 1;
    //     self.tx_fees += amt;

    //     let transfer = events::TransferNftEvent {
    //         action_id: self.action_cnt,
    //         chain_nonce,
    //         mint_with,
    //         to,
    //         amt,
    //         contract: token_contract,
    //         token_id,
    //     };

    //     env::log_str(&format!(
    //         r#"EVENT_JSON:{{ "type": "TransferUnique", "data": {} }}"#,
    //         serde_json::to_string(&transfer).unwrap()
    //     ))
    // }

    // /// Unfreeze NEP-171 token.
    // #[payable]
    // pub fn validate_unfreeze_nft(&mut self, data: UnfreezeNftData, sig_data: Vec<u8>) {
    //     require!(!self.paused, "paused");

    //     self.require_sig(
    //         data.action_id,
    //         data.try_to_vec().unwrap(),
    //         sig_data,
    //         b"ValidateUnfreezeNft",
    //     );

    //     common_nft::ext(env::current_account_id()).nft_transfer(
    //         env::signer_account_id(),
    //         data.token_id,
    //         None,
    //         None,
    //     );
    // }

    pub fn get_group_key(&self) -> [u8; 32] {
        self.group_key
    }

    pub fn is_whitelist(&self, contract_id: String) -> bool {
        self.whitelist.contains_key(&contract_id)
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}

// #[cfg(all(test, not(target_arch = "wasm32")))]
// mod tests {
//     use ed25519_dalek::{ExpandedSecretKey, Keypair};
//     use near_sdk::test_utils::VMContextBuilder;
//     use near_sdk::{testing_env, Balance};
//     use rand_core::OsRng;

//     use super::*;

//     const NEAR: u128 = 1000000000000000000000000;

//     fn set_context(predecessor: &str, amount: Balance) {
//         let mut builder = VMContextBuilder::new();
//         builder.predecessor_account_id(predecessor.parse().unwrap());
//         builder.attached_deposit(amount);
//         testing_env!(builder.build());
//     }

//     fn init_func(group_key: [u8; 32]) -> XpBridge {
//         XpBridge::initialize(group_key)
//     }

//     #[test]
//     fn init() {
//         set_context("admin", 1 * NEAR);

//         let mut csprng = OsRng {};
//         let kp = Keypair::generate(&mut csprng);
//         let group_key: [u8; 32] = kp.public.to_bytes();

//         let contract = init_func(group_key);

//         assert_eq!(group_key, contract.get_group_key());
//     }

//     // #[test]
//     // fn test_sig() {
//     //     // set_context("admin", 1 * NEAR);
//     //     let pk: ExpandedSecretKey = ExpandedSecretKey::from_bytes(&[
//     //         101, 99, 54, 99, 56, 50, 50, 52, 55, 100, 98, 98, 49, 53, 56, 57, 98, 101, 98, 48, 99,
//     //         102, 51, 57, 102, 52, 101, 55, 97, 50, 100, 97, 52, 50, 98, 102, 101, 102, 48, 102, 55,
//     //         100, 98, 97, 51, 49, 50, 97, 102, 51, 100, 102, 56, 98, 52, 56, 56, 97, 55, 51, 100,
//     //         55, 48, 55,
//     //     ])
//     //     .unwrap();

//     //     let gk = PublicKey::from(&pk);

//     //     let data = hex::decode("668a04000000000000000000000000000d00000078706e66742e746573746e65741a00000030783633313931376262306538336338656164383132656535640f00000069616d736b31372e746573746e65740106000000556e69616972019e0000004f6e652077697468207468652063616c6d2c2074686520556e6961697220697320612073706563696573207468617420656e6a6f797320746865207472616e7175696c6c697479206f6620656c65766174696f6e2e2041207472756520756e646572646f672c2074686520556e6961697220676976657320616e20656e64656172696e672070726573656e63652077697468206576657279206d6f76652e013e00000068747470733a2f2f6173736574732e706f6c6b616d6f6e2e636f6d2f696d616765732f556e696d6f6e735f5430364330324831304230344730302e6a7067000000000000000000").unwrap();

//     //     let transfer = TransferNftData::try_from_slice(&data).unwrap();

//     //     let sig = pk.sign(&data, &gk);

//     //     let group_key: [u8; 32] = gk.to_bytes();

//     //     let mut contract = init_func(group_key);
//     //     contract.require_sig(
//     //         297574,
//     //         data,
//     //         sig.to_bytes().to_vec(),
//     //         b"ValidateTransferNft",
//     //     );

//     //     assert_eq!(group_key, contract.get_group_key());
//     // }
//     #[test]
//     fn pause_unpause() {
//         set_context("admin", 1 * NEAR);

//         let mut csprng = OsRng {};
//         let kp = Keypair::generate(&mut csprng);
//         let group_key: [u8; 32] = kp.public.to_bytes();

//         let mut contract = init_func(group_key);

//         let data = PauseData { action_id: 1 };
//         let secret: ExpandedSecretKey = (&kp.secret).into();
//         let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

//         contract.validate_pause(data, sig.to_bytes().to_vec());

//         assert_eq!(true, contract.is_paused());

//         let data = UnpauseData { action_id: 2 };
//         let secret: ExpandedSecretKey = (&kp.secret).into();
//         let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

//         contract.validate_unpause(data, sig.to_bytes().to_vec());

//         assert_eq!(false, contract.is_paused());
//     }

//     #[test]
//     fn update_group_key() {
//         set_context("admin", 1 * NEAR);

//         let mut csprng = OsRng {};
//         let kp = Keypair::generate(&mut csprng);
//         let group_key: [u8; 32] = kp.public.to_bytes();

//         let mut contract = init_func(group_key);

//         let mut new_csprng = OsRng {};
//         let new_kp = Keypair::generate(&mut new_csprng);
//         let new_group_key: [u8; 32] = new_kp.public.to_bytes();

//         let data = UpdateGroupkeyData {
//             action_id: 3,
//             group_key: new_group_key,
//         };
//         let secret: ExpandedSecretKey = (&kp.secret).into();
//         let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

//         contract.validate_update_group_key(data, sig.to_bytes().to_vec());

//         assert_eq!(new_group_key, contract.get_group_key());
//     }

//     #[test]
//     fn freeze_unfreeze_nft() {
//         set_context("sender", 2 * NEAR);

//         let mut csprng = OsRng {};
//         let kp = Keypair::generate(&mut csprng);
//         let group_key: [u8; 32] = kp.public.to_bytes();

//         let mut contract = init_func(group_key);

//         contract.freeze_nft(
//             "test_nft".to_string(),
//             0x0,
//             "address_of_foreign".to_string(),
//             1 * NEAR,
//             "foreign_nft_contract".to_string(),
//         );

//         let data = UnfreezeNftData {
//             action_id: 1,
//             token_id: "test_nft".to_string(),
//         };
//         let secret: ExpandedSecretKey = (&kp.secret).into();
//         let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

//         contract.validate_unfreeze_nft(data, sig.to_bytes().to_vec());
//     }

//     #[test]
//     fn mint_burn_nft() {
//         set_context("sender", 2 * NEAR);

//         let mut csprng = OsRng {};
//         let kp = Keypair::generate(&mut csprng);
//         let group_key: [u8; 32] = kp.public.to_bytes();

//         let mut contract = init_func(group_key);

//         let data = WhitelistData {
//             action_id: 1,
//             mint_with: "test_nft".to_string(),
//         };
//         let secret: ExpandedSecretKey = (&kp.secret).into();
//         let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

//         contract.validate_whitelist(data, sig.to_bytes().to_vec());

//         let data = TransferNftData {
//             action_id: 2,
//             mint_with: "test_nft".to_string(),
//             token_id: "test_nft".to_string(),
//             owner_id: env::predecessor_account_id(),
//             token_metadata: TokenMetadata {
//                 title: Some("title".to_string()),
//                 description: Some("description".to_string()),
//                 media: None,
//                 media_hash: None,
//                 copies: None,
//                 issued_at: None,
//                 expires_at: None,
//                 starts_at: None,
//                 updated_at: None,
//                 extra: None,
//                 reference: None,
//                 reference_hash: None,
//             },
//         };
//         let secret: ExpandedSecretKey = (&kp.secret).into();
//         let sig = secret.sign(&(data.try_to_vec().unwrap()), &kp.public);

//         contract.validate_transfer_nft(data, sig.to_bytes().to_vec());

//         contract.withdraw_nft(
//             "test_nft".to_string(),
//             0x0,
//             "address_of_foreign".to_string(),
//             1 * NEAR,
//         );
//     }
// }
