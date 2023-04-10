use near_sdk::{
    env,
    serde::{Deserialize, Serialize},
    serde_json::{self},
    AccountId,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Event<'s, T> {
    pub event_type: &'s str,
    pub event: T,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CollectionCreated {
    pub mint_with: AccountId,
    pub name: String,
    pub symbol: String,
    pub bridge: AccountId,
    pub admin: AccountId,
    pub receiver: AccountId,
    pub collection_owner: AccountId,
    pub fee_numerator: u128,
}

impl CollectionCreated {
    fn to_json_string(&self) -> String {
        let event = Event {
            event: self,
            event_type: "CollectionCreated",
        };
        // Events cannot fail to serialize so fine to panic on error
        serde_json::to_string(&event)
            .ok()
            .unwrap_or_else(|| env::abort())
    }

    fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    pub fn emit(self) {
        env::log_str(&self.to_json_event_string());
    }
}
