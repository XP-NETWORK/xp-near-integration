use near_bigint::U256;
use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, Token, TokenId};
use near_sdk::{ext_contract, AccountId, Promise};

pub const TYOCTO: u128 = 1_000_000_000_000;
pub const TGAS: u64 = 1_000_000_000_000;

#[ext_contract(xpnft)]
pub trait XpNft {
    fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token;

    fn nft_burn(&mut self, token_id: TokenId, from: AccountId) -> Promise;

    fn nft_token(&self, token_id: TokenId) -> Option<Token>;
}

#[ext_contract(common_nft)]
pub trait CommonNft {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
}

#[ext_contract(currency_data_oracle)]
pub trait CurrencyDataOracle {
    fn estimate_fees(&self, from: u16, to: u16) -> Option<U256>;
}
