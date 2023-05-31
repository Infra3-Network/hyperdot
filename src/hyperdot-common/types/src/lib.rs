use hyperdot_common_macros::ToParams;
use serde::Serialize;
use serde::Deserialize;


/// A block request type
#[derive(Serialize, Deserialize, ToParams)]
pub struct WriteBlockHeaderRequest {
    pub block_number: u64,
    pub block_hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub state_root: Vec<u8>,
    pub extrinsics_root: Vec<u8>,
}

/// A block request type
#[derive(Serialize, Deserialize, Clone)]
pub struct WriteBlockHeaderResponse;