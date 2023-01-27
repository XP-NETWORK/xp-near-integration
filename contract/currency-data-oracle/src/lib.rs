use ed25519_compact::{PublicKey, Signature};
use near_bigint::U256;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require};
use sha2::{Digest, Sha512};
use std::collections::HashMap;

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdateData {
    new_data: HashMap<u16, U256>, // chain_nonce -> price
    action_id: U256,              // random action id
}

#[derive(Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdateGroupkeyData {
    action_id: U256,
    group_key: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CurrencyData {
    pub from_chain_price: U256,
    pub to_chain_price: U256,
    pub from_chain_decimal: U256,
    pub to_chain_decimal: U256,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct CurrencyDataOracle {
    price_data: HashMap<u16, U256>,        // chain_nonce -> price
    group_key: [u8; 32],                   // group key for signature verification
    consumed_actions: HashMap<u128, bool>, // action_id -> bool
    decimals: HashMap<u16, U256>,          // chain_nonce -> decimal value
    chain_tx_fee_data: HashMap<u16, U256>, // chain_nonce -> tx fee
    other_fees: HashMap<u16, U256>, // chain_nonce -> other fees (for future proofing, extra fee in chains)
}

#[near_bindgen]
impl CurrencyDataOracle {
    /// Initializes the contract with the provided group key.
    /// Also sets the initial decimal, and price data values, and empty hashmaps
    /// for other contract state variables.
    #[init]
    pub fn initialize(
        group_key: [u8; 32],
        decimals: HashMap<u16, U256>,
        price_data: HashMap<u16, U256>,
        chain_tx_fee_data: HashMap<u16, U256>,
        other_fees: HashMap<u16, U256>,
    ) -> Self {
        assert!(
            env::current_account_id() == env::predecessor_account_id(),
            "Unauthorized"
        );

        assert!(!env::state_exists(), "Already initialized");

        Self {
            price_data,
            group_key,
            decimals,
            chain_tx_fee_data,
            other_fees,
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
    pub fn validate_update_prices(&mut self, data: UpdateData, sig_data: Vec<u8>) {
        self.require_sig(
            data.action_id.as_u128(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"UpdateData",
        );

        self.price_data.extend(data.new_data.iter());
    }

    /// Updates the price data in the state of the contract.
    pub fn validate_update_tx_fees(&mut self, data: UpdateData, sig_data: Vec<u8>) {
        self.require_sig(
            data.action_id.as_u128(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"UpdateData",
        );

        self.chain_tx_fee_data.extend(data.new_data.iter());
    }

    /// Updates the price data in the state of the contract.
    pub fn validate_update_other_fees(&mut self, data: UpdateData, sig_data: Vec<u8>) {
        self.require_sig(
            data.action_id.as_u128(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"UpdateData",
        );

        self.other_fees.extend(data.new_data.iter());
    }

    /// Updates the decimal data in the state of the contract.
    pub fn validate_update_decimal(&mut self, data: UpdateData, sig_data: Vec<u8>) {
        self.require_sig(
            data.action_id.as_u128(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"AddDecimalData",
        );

        self.decimals.extend(data.new_data.iter());
    }

    /// Get Price Data for the given nonces.
    pub fn get_price_data(&self, from_nonce: u16, to_nonce: u16) -> HashMap<u16, U256> {
        let mut res = HashMap::new();
        let from = self.price_data.get(&from_nonce).unwrap();
        let to = self.price_data.get(&to_nonce).unwrap();
        res.insert(from_nonce, *from);
        res.insert(to_nonce, *to);
        res
    }

    /// Get Price Data for the given nonces.
    pub fn get_decimal_data(&self, from_nonce: u16, to_nonce: u16) -> HashMap<u16, U256> {
        let mut res = HashMap::new();
        let from = self.decimals.get(&from_nonce).unwrap();
        let to = self.decimals.get(&to_nonce).unwrap();
        res.insert(from_nonce, *from);
        res.insert(to_nonce, *to);
        res
    }

    /// Get Full Currency Data for the given nonces.
    /// Includes price and decimal data for both chains.
    pub fn get_currency_data(&self, from_nonce: u16, to_nonce: u16) -> CurrencyData {
        CurrencyData {
            from_chain_price: *self.price_data.get(&from_nonce).expect(&format!(
                "No price data available for chain nonce : {}",
                from_nonce
            )),
            to_chain_price: *self.price_data.get(&to_nonce).expect(&format!(
                "No price data available for chain nonce : {}",
                to_nonce
            )),
            from_chain_decimal: *self.decimals.get(&from_nonce).expect(&format!(
                "No decimal data available for chain nonce : {}",
                to_nonce
            )),
            to_chain_decimal: *self.decimals.get(&from_nonce).expect(&format!(
                "No decimal data available for chain nonce : {}",
                to_nonce
            )),
        }
    }

    /// Updates the group key in the state of the contract.
    pub fn validate_update_group_key(&mut self, data: UpdateGroupkeyData, sig_data: Vec<u8>) {
        self.require_sig(
            data.action_id.as_u128(),
            data.try_to_vec().unwrap(),
            sig_data,
            b"SetGroupKey",
        );

        self.group_key = data.group_key;
    }

    /// Estimates the fees for the given chains.
    /// Returns the estimated fees in to_chain currency and decimals.
    /// The fees are calculated as follows:
    /// 1. Get the tx fee for the to_chain.
    /// 2. Get the conversion rate for the to_chain.
    /// 3. Multiply the tx fee with the conversion rate.
    /// 4. Now in this fee (in USD), add our commission fees.
    /// 5. Now convert this fee back into from_chain currency.
    pub fn estimate_fees(&self, from: u16, to: u16) -> Option<U256> {
        let Some(from_dec) = self.decimals.get(&from) else {
            env::log_str("Failed to get decimal for from chain");
            return None;
        };
        let Some(to_dec) = self.decimals.get(&to) else {
            env::log_str("Failed to get decimal for to chain");
            return None;
        };

        let Some(from_conv_rate) = self.price_data.get(&from) else {
            env::log_str("Failed to get conv data for from chain");
            return None;
        };

        let Some(to_conv_rate) = self.price_data.get(&to) else {
            env::log_str("Failed to get conv data for to chain");
            return None;
        };

        let Some(tx_fee) = self.chain_tx_fee_data.get(&to) else {
            env::log_str("Failed to get tx fee data for to chain");
            return None;
        };

        let to_tx_fee = tx_fee + self.other_fees.get(&to).unwrap_or(&U256::zero());

        let fee_in_usd = (to_tx_fee * to_conv_rate) / to_dec;

        let fee_in_usd_with_commission = fee_in_usd + (to_dec / 2); // + 0.5 USD

        let fee_in_from_currency =
            (fee_in_usd_with_commission * from_dec) / from_conv_rate * (from_dec / to_dec);

        Some(fee_in_from_currency)
    }

    /// Get the group key from the state of the contract.
    pub fn get_group_key(&self) -> [u8; 32] {
        self.group_key
    }

    /// Gets the price data from the state of the contract for all the chains.
    pub fn get_all_price_data(&self) -> HashMap<u16, U256> {
        self.price_data.clone()
    }

    /// Gets the decimal data from the state of the contract for all the chains.
    pub fn get_all_decimal_data(&self) -> HashMap<u16, U256> {
        self.price_data.clone()
    }

    /// Gets the tx fee data from the state of the contract for all the chains.
    pub fn get_all_tx_fee_data(&self) -> HashMap<u16, U256> {
        self.chain_tx_fee_data.clone()
    }

    /// Gets the tx fee data from the state of the contract for all the chains.
    pub fn get_all_other_fee_data(&self) -> HashMap<u16, U256> {
        self.other_fees.clone()
    }

    /// Encodes the UpdateData struct into a vector of bytes.
    /// This should be done in the client side but i cant find
    /// a way to acoomplish this with borsh-ts so we do it here.
    pub fn encode_update_data(&self, new_data: HashMap<u16, U256>, action_id: U256) -> Vec<u8> {
        let data = UpdateData {
            new_data,
            action_id,
        };
        data.try_to_vec().unwrap()
    }
    /// Encodes the UpdateGroupkeyData struct into a vector of bytes.
    /// This should be done in the client side but i cant find
    /// a way to acoomplish this with borsh-ts so we do it here.
    pub fn encode_update_group_key(&self, group_key: [u8; 32], action_id: U256) -> Vec<u8> {
        let data = UpdateGroupkeyData {
            group_key,
            action_id,
        };
        data.try_to_vec().unwrap()
    }
}
