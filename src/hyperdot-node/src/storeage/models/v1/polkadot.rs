use serde::Serialize;
#[derive(Serialize)]
pub struct Block {
    pub block_number: i64,
    pub block_hash: String,
    pub parent_hash: String,
    pub state_root: String,
    pub extrinsics_root: String,
}

