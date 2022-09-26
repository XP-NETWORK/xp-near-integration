use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, Token, TokenId};
use near_sdk::{ext_contract, AccountId, Promise};

pub const TYOCTO: u128 = 1_000_000_000_000;

#[ext_contract(xpnft)]
pub trait XpNft {
    fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token;

    fn nft_burn(&mut self, token_id: TokenId, from: AccountId) -> Promise;
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
