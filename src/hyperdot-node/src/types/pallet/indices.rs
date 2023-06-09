use serde::Serialize;
use serde::Deserialize; 

use crate::types::pallet::utils::AccountId32;

#[derive(Serialize, Deserialize)]
pub enum Event {
    IndexAssigned {
        who: AccountId32,
        index: u32,
    },
    IndexFreed { index: u32 },
    IndexFrozen {
        index: u32,
        who: AccountId32,
    },
}