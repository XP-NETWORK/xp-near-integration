use near_sdk::collections::{UnorderedSet};

use crate::*;

pub trait NonFungibleTokenCore {
    //calculates the payout for a token given the passed in balance. This is a view method
    fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout;

    //transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance.
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    //calculates the payout for a token given the passed in balance. This is a view method
    fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout {
        //get the token object
        let token = self.tokens_by_id.get(&token_id).expect("No token");

        //get the owner of the token
        let owner_id = token.owner_id;
        //keep track of the total perpetual royalties
        let mut total_perpetual = 0;
        //get the u128 version of the passed in balance (which was U128 before)
        let balance_u128 = u128::from(balance);
        //keep track of the payout object to send back
        let mut payout_object = Payout {
            payout: HashMap::new(),
        };
        //get the royalty object from token
        let royalty = token.royalty;

        //make sure we're not paying out to too many people (GAS limits this)
        assert!(
            royalty.len() as u32 <= max_len_payout,
            "Market cannot payout to that many receivers"
        );

        //go through each key and value in the royalty object
        for (k, v) in royalty.iter() {
            //get the key
            let key = k.clone();
            //only insert into the payout if the key isn't the token owner (we add their payout at the end)
            if key != owner_id {
                //
                payout_object
                    .payout
                    .insert(key, royalty_to_payout(*v, balance_u128));
                total_perpetual += *v;
            }
        }

        // payout to previous owner who gets 100% - total perpetual royalties
        payout_object.payout.insert(
            owner_id,
            royalty_to_payout(10000 - total_perpetual, balance_u128),
        );

        //return the payout object
        payout_object
    }

    //transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance.
    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        //assert that the user attached 1 yocto NEAR for security reasons
        assert_one_yocto();
        //get the sender ID
        let sender_id = env::predecessor_account_id();
        //transfer the token to the passed in receiver and get the previous token object back
        let previous_token =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, Some(approval_id), memo);

        //refund the previous token owner for the storage used up by the previous approved account IDs
        refund_approved_account_ids(
            previous_token.owner_id.clone(),
            &previous_token.approved_account_ids,
        );

        //get the owner of the token
        let owner_id = previous_token.owner_id;
        //keep track of the total perpetual royalties
        let mut total_perpetual = 0;
        //get the u128 version of the passed in balance (which was U128 before)
        let balance_u128 = u128::from(balance);
        //keep track of the payout object to send back
        let mut payout_object = Payout {
            payout: HashMap::new(),
        };
        //get the royalty object from token
        let royalty = previous_token.royalty;

        //make sure we're not paying out to too many people (GAS limits this)
        assert!(
            royalty.len() as u32 <= max_len_payout,
            "Market cannot payout to that many receivers"
        );

        //go through each key and value in the royalty object
        for (k, v) in royalty.iter() {
            //get the key
            let key = k.clone();
            //only insert into the payout if the key isn't the token owner (we add their payout at the end)
            if key != owner_id {
                //
                payout_object
                    .payout
                    .insert(key, royalty_to_payout(*v, balance_u128));
                total_perpetual += *v;
            }
        }

        // payout to previous owner who gets 100% - total perpetual royalties
        payout_object.payout.insert(
            owner_id.clone(),
            royalty_to_payout(10000 - total_perpetual, balance_u128),
        );

        // Update the mappings of previous owner.
        let owner_tokens = &mut self
            .tokens_per_owner
            .get(&owner_id)
            .unwrap_or_else(|| UnorderedSet::new(b"s"));

        if owner_tokens.len() > 0 {
            owner_tokens.remove(&token_id);
            if owner_tokens.is_empty() {
                self.tokens_per_owner.remove(&owner_id);
            } else {
                self.tokens_per_owner.insert(&owner_id, &owner_tokens);
            }
        }

        // Update the mappings of new owner.
        self.owner_by_id.insert(&token_id, &receiver_id);

        let if_exists_new_owner_tokens = &mut self
            .tokens_per_owner
            .get(&receiver_id)
            .unwrap_or_else(|| UnorderedSet::new(b"s"));

        if if_exists_new_owner_tokens.len() > 0 {
            if_exists_new_owner_tokens.insert(&token_id);
            self.tokens_per_owner
                .insert(&receiver_id, &if_exists_new_owner_tokens);
        } else {
            let mut new_owner_tokens = UnorderedSet::new(b"s");
            new_owner_tokens.insert(&token_id);
            self.tokens_per_owner
                .insert(&receiver_id, &new_owner_tokens);
        }

        //return the payout object
        payout_object
    }
}
