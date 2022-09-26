use near_contract_standards::non_fungible_token::events::{NftBurn, NftMint};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, Promise, PromiseOrValue, PanicOnDefault};

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn initialize(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Unauthorized"
        );
        let token = self.tokens.internal_mint_with_refund(
            token_id,
            token_owner_id,
            Some(token_metadata),
            Some(env::predecessor_account_id()),
        );

        NftMint {
            owner_id: &token.owner_id,
            token_ids: &[&token.token_id],
            memo: None,
        }
        .emit();

        token
    }

    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId, from: AccountId) -> Promise {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Unauthorized"
        );

        let owner = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .expect("unknown token id");

        if owner != from {
            env::panic_str("owner is not who we expected it was")
        }

        let storage_used = env::storage_usage();

        // A lot of moving parts here.. code reviewers.. did I get it
        // all?  Hard to believe nobody has implemented burn in the
        // standard SDK.  Googling around found me some other NFT
        // contracts that tried to implement it but they didn't get
        // the storage management correct.

        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            // owner_tokens should always exist, so call `unwrap` without guard
            let mut owner_tokens = tokens_per_owner.get(&from).unwrap_or_else(|| {
                env::panic_str("Unable to access tokens per owner in unguarded call.")
            });
            owner_tokens.remove(&token_id);
            if owner_tokens.is_empty() {
                tokens_per_owner.remove(&from);
            } else {
                tokens_per_owner.insert(&from, &owner_tokens);
            }
        }

        self.tokens.owner_by_id.remove(&token_id);

        if let Some(next_approval_id_by_id) = &mut self.tokens.next_approval_id_by_id {
            next_approval_id_by_id.remove(&token_id);
        }
        if let Some(approvals_by_id) = &mut self.tokens.approvals_by_id {
            approvals_by_id.remove(&token_id);
        }
        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.remove(&token_id);
        }

        NftBurn {
            owner_id: &from,
            token_ids: &[&token_id],
            authorized_id: Some(&env::predecessor_account_id()),
            memo: None,
        }
        .emit();

        let storage_freed = storage_used - env::storage_usage();

        Promise::new(env::predecessor_account_id())
            .transfer(storage_freed as u128 * env::storage_byte_cost())
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}
