use hyperdot_common_macros::ToParams;
use serde::Deserialize;
use serde::Serialize;

pub mod pallet;

/// A block request type
#[derive(Clone, Serialize, Deserialize)]
pub struct BlockHeaderDescribe {
    pub block_number: u64,
    pub block_hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub state_root: Vec<u8>,
    pub extrinsics_root: Vec<u8>,
}

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

/// Raw event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToParams)]
pub struct Event {
    pub block_hash: Vec<u8>,
    pub block_number: u64,
    pub block_time: u64, // TODO: not used currently.
    pub extrinsic_hash: Vec<u8>,
    pub data: Vec<u8>,
    pub index: u32,
    pub topic0: Vec<u8>,
    pub topic1: Vec<u8>,
    pub topic2: Vec<u8>,
    pub topic3: Vec<u8>,
    pub topic4: Vec<u8>,
    pub phase: EventPhase,
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let topics = {
            let mut nums = 0;
            if !self.topic0.is_empty() {
                nums += 1;
            }

            if !self.topic1.is_empty() {
                nums += 1;
            }

            if !self.topic2.is_empty() {
                nums += 1;
            }

            if !self.topic3.is_empty() {
                nums += 1;
            }

            if !self.topic4.is_empty() {
                nums += 1;
            }
            nums
        };
        write!(f, "Event\n")?;
        // write!(f, "  block_hash: {:?}\n", self.block_hash)?; // TODO hash string is better way
        write!(f, "     block_number: {}\n", self.block_number)?;
        write!(f, "     index: {}\n", self.index)?;
        write!(f, "     topics: {}\n", topics)?;
        write!(f, "     phase: {:?}\n", self.phase)?;
        write!(f, "\n")
    }
}

/// Decoded event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToParams)]
pub struct EventDecode {
    pub block_hash: Vec<u8>,
    pub block_number: u64,
    pub block_time: u64, // TODO: not used currently.
    pub extrinsic_hash: Vec<u8>,
    pub index: u32,
    pub phase: EventPhase,
    pub pallet_name: String,
    pub pallet_index: u8,
    // TODO signature
}

use pallet::balance::event::Transfer;
use pallet::balance::event::Withdraw;
use pallet::system::event::ExtrinsicFailed;
use pallet::system::event::ExtrinsicSuccess;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum ExtrinsicEventDescribe {
    Transfer(Transfer),
    Withdraw(Withdraw),
    ExtrinsicSuccess(ExtrinsicSuccess),
    ExtrinsicFailed(ExtrinsicFailed),
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
    pub events: Vec<ExtrinsicEventDescribe>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockDescribe {
    pub header: BlockHeaderDescribe,
    // pub events: Vec<Event>,
    // pub event_decodes: Vec<EventDecode>,
    pub extrinsics: Vec<ExtrinsicDescribe>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, ToParams)]
pub struct WriteBlockRequest {
    pub blocks: Vec<BlockDescribe>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, ToParams)]
pub struct WriteBlockResponse {}

#[cfg(test)]
mod tests {
    use super::Event;
    use super::EventPhase;

    #[test]
    fn event_encode_deode() {
        let ev1 = Event {
            block_hash: vec![0_u8; 2],
            block_number: 1,
            block_time: 0,
            extrinsic_hash: vec![],
            data: vec![],
            index: 0,
            topic0: vec![],
            topic1: vec![],
            topic2: vec![],
            topic3: vec![],
            topic4: vec![],
            phase: EventPhase::ApplyExtrinsic(1),
        };
        let raw_ev = r#"{
          "block_hash": [
            0,
            0
          ],
          "block_number": 1,
          "block_time": 0,
          "extrinsic_hash": [],
          "data": [],
          "index": 0,
          "topic0": [],
          "topic1": [],
          "topic2": [],
          "topic3": [],
          "topic4": [],
          "phase": {
            "ApplyExtrinsic": 1
          }
        }"#;
        let ev2 = serde_json::from_str(raw_ev).unwrap();
        assert_eq!(ev1, ev2);
    }
}
