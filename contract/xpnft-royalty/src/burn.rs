use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId, from: AccountId) -> Promise {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Unauthorized");

        let owner = self.owner_by_id.get(&token_id).expect("unknown token id");

        if owner != from {
            env::panic_str("owner is not who we expected it was")
        }

        // A lot of moving parts here.. code reviewers.. did I get it
        // all?  Hard to believe nobody has implemented burn in the
        // standard SDK.  Googling around found me some other NFT
        // contracts that tried to implement it but they didn't get
        // the storage management correct.

        let owner_tokens = &mut self.tokens_per_owner.get(&from).unwrap_or_else(|| {
            env::panic_str("Unable to access tokens per owner in unguarded call.")
        });

        if owner_tokens.len() > 0 {
            owner_tokens.remove(&token_id);
            if owner_tokens.is_empty() {
                self.tokens_per_owner.remove(&from);
            } else {
                self.tokens_per_owner.insert(&from, &owner_tokens);
            }
        }

        self.owner_by_id.remove(&token_id);

        // self.nft_revoke_all(token_id);

        // Construct the mint log as per the events standard.
        let nft_burn_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftBurn(vec![NftBurnLog {
                // Owner of the token.
                owner_id: owner.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo: None,
                authorized_id: Some(env::predecessor_account_id()),
            }]),
        };

        let next_approval_id_by_id = &mut self.tokens_by_id.get(&token_id).unwrap();
        next_approval_id_by_id.next_approval_id = 0;
        next_approval_id_by_id.approved_account_ids.clear();
        self.tokens_by_id.remove(&token_id);
        self.token_metadata_by_id.remove(&token_id);

        // Log the serialized json.
        env::log_str(&nft_burn_log.to_string());

        Promise::new(env::predecessor_account_id())
    }
}
