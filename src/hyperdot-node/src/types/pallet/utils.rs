use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct AccountId32(pub [u8; 32]);
