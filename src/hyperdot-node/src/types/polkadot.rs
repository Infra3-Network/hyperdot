use hyperdot_common_macros::ToParams;
use serde::Deserialize;
use serde::Serialize;


/// A block request type
#[derive(Serialize, Deserialize, Clone)]
pub struct WriteBlockHeaderResponse;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EventPhase {
    /// Applying an extrinsic.
    ApplyExtrinsic(u32),
    /// Finalizing the block.
    Finalization,
    /// Initializing the block.
    Initialization,
}

impl EventPhase {
    pub fn to_string(&self) -> String {
        match *self {
            EventPhase::ApplyExtrinsic(value) => format!("ApplyExtrinsic({})", value),
            EventPhase::Finalization => "Finalization".to_string(),
            EventPhase::Initialization => "Initialization".to_string(),
        }
    }
}

/// Raw event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToParams)]
pub struct EventDescribe {
    /// Unindexed data containing further information on the event
    pub data: Vec<u8>,
    /// What index is this event in the stored events for this block.
    pub index: u32,
    /// The hash of topics.
    pub topics: Vec<Vec<u8>>,
    /// The phase of event.
    pub phase: EventPhase,
    /// The pallet name of event.
    pub pallet_name: String,
    /// The pallet index of event.
    pub pallet_index: u8,
    pub root_bytes: Vec<u8>,
    /// The hash of extrinsic.
    pub extrinsic_hash: Vec<u8>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtrinsicDescribe {
    /// The index of the extrinsic in the block.
    pub index: u32,
    /// The pallet index.
    pub pallet_index: u8,
    /// The name of the pallet from whence the extrinsic originated.
    pub pallet_name: String,
    /// The hash of extrinsic.
    pub hash: Vec<u8>,

    /// The root call bytes of extrinsic.
    pub root_call_bytes: Vec<u8>,

    /// pub events: Vec<ExtrinsicEventDescribe>,
    pub events: Vec<EventDescribe>,
}

/// A block request type
#[derive(Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub block_number: u64,
    pub block_hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub state_root: Vec<u8>,
    pub extrinsics_root: Vec<u8>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    /// The decoded extrinsics record key of extrinsic in block.
    pub extrinsics: Vec<ExtrinsicDescribe>,
}

// header + extrinsics
// extrinsics -> events
//  events
//      per event -> EventDetails + RootEvent Raw data
//  per extrinsics -> ExtrinsicDetails + RootCall raw data


