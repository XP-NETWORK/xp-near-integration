use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_sdk::{ext_contract, AccountId};

#[ext_contract(xpnft_royalty)]
pub trait XpNftRoyalty {
    fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self;
}
