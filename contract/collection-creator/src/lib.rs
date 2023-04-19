use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;

use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Gas, PromiseError};
use near_sdk::{PanicOnDefault, Promise};
pub mod events;
mod external;
pub use crate::events::*;
use crate::external::xpnft_royalty;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct CollectionCreator {
    collections: LookupMap<String, AccountId>,
    allowed_accounts: Vec<AccountId>,
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NewCollectionParams {
    pub account_id: AccountId,
    pub owner_id: AccountId,
    pub receiver: AccountId,
    pub bridge: AccountId,
    pub name: String,
    pub collection_identifier: String,
    pub symbol: String,
    pub base_uri: String,
}

const CODE: &[u8] = include_bytes!("assets/xpnft_royalty.wasm");

#[near_bindgen]
impl CollectionCreator {
    /// Initializes the contract
    #[init]
    pub fn initialize(params: Vec<AccountId>) -> Self {
        assert!(
            env::current_account_id() == env::predecessor_account_id(),
            "Unauthorized"
        );

        Self {
            collections: LookupMap::new(b"c"),
            allowed_accounts: params,
        }
    }

    pub fn check_collection(&self, identifier: String) -> AccountId {
        self.collections
            .get(&identifier)
            .expect("No Collection Exists")
    }

    pub fn deploy_collection(&self, params: NewCollectionParams) {
        let allowed_accts = self.allowed_accounts.clone();
        let result = allowed_accts
            .iter()
            .find(|&s| *s == env::signer_account_id());
        match result {
            Some(_) => println!("Account found"),
            None => env::panic_str("Caller is not in the allowed accounts"),
        }
        Promise::new(params.account_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(5_000_000_000_000_000_000_000_000) // 5e24yN, 5N
            .deploy_contract(CODE.to_vec())
            .then(
                xpnft_royalty::ext(params.account_id.clone())
                    .with_static_gas(Gas(4_000_000_000_000))
                    .new(
                        params.owner_id.clone(),
                        NFTContractMetadata {
                            spec: "nft-1.0.0".to_string(),
                            name: params.name.clone(),
                            symbol: params.symbol.clone(),
                            icon: None,
                            base_uri: Some(params.base_uri.to_string()),
                            reference: None,
                            reference_hash: None,
                        },
                    ),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5_000_000_000_000))
                    .handle_successful_deploy_init_callback(params.clone()),
            );
    }

    #[private]
    pub fn handle_successful_deploy_init_callback(
        &mut self,
        params: NewCollectionParams,
        #[callback_result] result: Result<(), PromiseError>,
    ) {
        match result {
            Ok(_) => {
                CollectionCreated {
                    admin: params.owner_id.clone(),
                    mint_with: params.account_id.clone(),
                    name: params.name,
                    symbol: params.symbol,
                    bridge: params.bridge,
                    collection_owner: params.owner_id.clone(),
                    fee_numerator: 0,
                    receiver: params.receiver,
                    base_uri: params.base_uri,
                }
                .emit();
                self.collections
                    .insert(&params.collection_identifier, &params.account_id);
            }

            Err(e) => env::panic_str(&format!("Failed to Deploy and Init: {:?}", e)),
        }
    }
}
