use crate::*;
use near_sdk::{ext_contract, AccountId, Promise};
use std::collections::HashMap;

pub const TYOCTO: u128 = 1_000_000_000_000;
pub const TGAS: u64 = 1_000_000_000_000;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    //token ID
    pub token_id: TokenId,
    //owner of the token
    pub owner_id: AccountId,
    //token metadata
    pub metadata: TokenMetadata,
    //list of approved account IDs that have access to transfer the token. This maps an account ID to an approval ID
    pub approved_account_ids: HashMap<AccountId, u64>,
    //keep track of the royalty percentages for the token in a hash map
    pub royalty: HashMap<AccountId, u32>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    //owner of the token
    pub owner_id: AccountId,
    //list of approved account IDs that have access to transfer the token. This maps an account ID to an approval ID
    pub approved_account_ids: HashMap<AccountId, u64>,
    //the next approval ID to give out. 
    pub next_approval_id: u64,
    //keep track of the royalty percentages for the token in a hash map
    pub royalty: HashMap<AccountId, u32>,
}

#[ext_contract(xpnft_royalty)]
pub trait XpNft {
    fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
        //we add an optional parameter for perpetual royalties
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) -> Token;

    fn nft_burn(&mut self, token_id: TokenId, from: AccountId) -> Promise;

    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
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
