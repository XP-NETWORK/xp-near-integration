use crate::*;
use near_sdk::{ext_contract, AccountId, Balance, Promise};
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

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
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

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MultiToken {
    pub token_id: String,
    pub owner_id: AccountId,
    /// Total amount generated
    pub supply: u128,
    pub metadata: Option<TokenMetadata>,
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

#[ext_contract(xpnft_erc1155)]
pub trait XpMt {
    fn mt_mint(
        &mut self,
        token_owner_id: AccountId,
        token_metadata: Vec<TokenMetadata>,
        supply: Vec<Balance>,
    ) -> MultiToken;

    fn mt_burn(
        &mut self,
        token_ids: Vec<TokenId>,
        token_amts: Vec<u128>,
        from: AccountId,
    ) -> Promise;

    fn mt_token(&self, token_ids: Vec<TokenId>) -> Option<MultiToken>;
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

#[ext_contract(common_mt)]
pub trait CommonMt {
    fn mt_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        amount: u128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
    );

    fn mt_batch_transfer(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<u128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
    );
}
