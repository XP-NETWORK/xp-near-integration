use ed25519_compact::{PublicKey, Signature};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require};
use sha2::{Digest, Sha512};
use std::collections::HashMap;

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdatePriceData {
    price: HashMap<u16, U128>, // chain_nonce -> price
    action_id: U128, // random action id
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AddDecimalData {
    nonce: u16, // nonce of the chain
    decimal: U128, // decimal value in the form of 1e18 for example
    action_id: U128, // random action id
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct CurrencyDataOracle {
    price_data: HashMap<u16, U128>,  // chain_nonce -> price
    group_key: [u8; 32],            // group key for signature verification
    consumed_actions: HashMap<u128, bool>, // action_id -> bool
    decimals: HashMap<u16, U128>, // chain_nonce -> decimal value
}

#[near_bindgen]
impl CurrencyDataOracle {
    /// Initializes the contract with the provided group key.
    /// Also sets the initial decimal, and price data values, and empty hashmaps
    /// for other contract state variables.
    #[init]
    pub fn initialize(group_key: [u8; 32], decimals: HashMap<u16, U128>, initial_price_data: HashMap<u16, U128>) -> Self {
        assert!(
            env::current_account_id() == env::predecessor_account_id(),
            "Unauthorized"
        );

        assert!(!env::state_exists(), "Already initialized");

        Self {
            price_data: initial_price_data,
            group_key,            
            decimals,
            consumed_actions: HashMap::new(),
        }
    }

    /// Ed25519 Signature verification logic.
    /// Signature check for contract state updating actions.
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

    /// Updates the price data in the state of the contract.
    pub fn validate_update_prices(&mut self, data: UpdatePriceData, sig_data: Vec<u8>) {
        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"UpdatePriceData",
        );

        self.price_data.extend(data.price.iter());
    }

    /// Updates the decimal data in the state of the contract.
    pub fn validate_add_decimal(&mut self, data: AddDecimalData, sig_data: Vec<u8>) {
        self.require_sig(
            data.action_id.into(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"AddDecimalData",
        );

        self.decimals.insert(data.nonce, data.decimal);
    }

    /// Get Price Data for the given nonces.
    pub fn get_price_data(&self, from_nonce: u16, to_nonce: u16) -> HashMap<u16, U128> {
        let mut res = HashMap::new();
        let from = self.price_data.get(&from_nonce).unwrap();
        let to = self.price_data.get(&to_nonce).unwrap();
        res.insert(from_nonce, *from);
        res.insert(to_nonce, *to);
        res
    }

    /// Get Price Data for the given nonces.
    pub fn get_decimal_data(&self, from_nonce: u16, to_nonce: u16) -> HashMap<u16, U128> {
        let mut res = HashMap::new();
        let from = self.decimals.get(&from_nonce).unwrap();
        let to = self.decimals.get(&to_nonce).unwrap();
        res.insert(from_nonce, *from);
        res.insert(to_nonce, *to);
        res
    }

    pub fn encode_update_price_data(&self, price: HashMap<u16, U128>, action_id: U128) -> Vec<u8> {
        let data = UpdatePriceData {
            price,
            action_id,
        };
        data.try_to_vec().unwrap()
    }
}
