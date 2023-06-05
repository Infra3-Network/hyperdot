use hyperdot_common_macros::ToParams;
use serde::Deserialize;
use serde::Serialize;

pub mod pallet;



/// A block request type
#[derive(Serialize, Deserialize, ToParams)]
pub struct WritableBlockHeader {
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

pub struct WritableBlock {
    pub header: WritableBlockHeader,
    pub events: Vec<Event>,
    pub event_decodes: Vec<EventDecode>,
    pub extrinsics: Vec<WritableExtrinsic>,
}

use pallet::balance::event::Transfer;
use pallet::balance::event::Withdraw;
use pallet::system::event::ExtrinsicFailed;
use pallet::system::event::ExtrinsicSuccess;

pub enum WritableExtrinsicEvent {
    Transfer(Transfer),
    Withdraw(Withdraw),
    ExtrinsicSuccess(ExtrinsicSuccess),
    ExtrinsicFailed(ExtrinsicFailed),
}

pub struct WritableExtrinsic {
    pub events: Vec<WritableExtrinsicEvent>,
}



pub mod runtime {
    // use std::fmt::Debug;

    // use serde::de::DeserializeOwned;
    // use serde::Deserialize;
    // use serde::Serialize;

    //#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    //pub enum MultiAddress<AccountId, AccountIndex>
    //{
    //    /// It's an account ID (pubkey).
    //    Id(AccountId),
    //    /// It's an account index.
    //    Index(AccountIndex),
    //    /// It's some arbitrary raw bytes.
    //    Raw(Vec<u8>),
    //    /// It's a 32 byte representation.
    //    Address32([u8; 32]),
    //    /// Its a 20 byte representation.
    //    Address20([u8; 20]),
    //}
}

// pub mod pallets {
//     pub mod balance {
//         use std::fmt::Debug;

//         use serde::de::DeserializeOwned;
//         use serde::Deserialize;
//         use serde::Serialize;
//         use subxt::utils::H256;

//         use super::runtime;

//         #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
//         pub struct Transfer {
//             pub block_hash: Vec<u8>,
//             pub block_number: u64,
//             pub block_time: u64, // TODO: not used currently.
//             pub extrinsic_hash: Vec<u8>,
//             pub index: u32,
//             pub from: [u8; 32],
//             pub to: [u8; 32],
//             pub amount: u128,
//             pub success: bool, // streaming?
//         }

//         impl std::fmt::Display for Transfer {
//             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "block_number: {}\n", self.block_number)?;
//                 write!(
//                     f,
//                     "block_hash: {:?}\n",
//                     H256::from_slice(self.block_hash.as_ref())
//                 )?;
//                 write!(
//                     f,
//                     "extrinsic_hash: {:?}\n",
//                     H256::from_slice(self.extrinsic_hash.as_ref())
//                 )?;
//                 write!(f, "index: {}\n", self.index)?;
//                 write!(
//                     f,
//                     "transfer {:?} => {:?}, {}\n",
//                     H256::from(&self.from),
//                     H256::from(&self.to),
//                     self.amount
//                 )
//             }
//         }

//         #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
//         pub struct Withdraw {
//             pub block_hash: Vec<u8>,
//             pub block_number: u64,
//             pub block_time: u64, // TODO: not used currently.
//             pub extrinsic_hash: Vec<u8>,
//             pub index: u32,
//             pub who: [u8; 32],
//             pub amount: u128,
//             pub success: bool,
//         }

//         impl std::fmt::Display for Withdraw {
//             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//                 write!(f, "block_number: {}\n", self.block_number)?;
//                 write!(
//                     f,
//                     "block_hash: {:?}\n",
//                     H256::from_slice(self.block_hash.as_ref())
//                 )?;
//                 write!(
//                     f,
//                     "extrinsic_hash: {:?}\n",
//                     H256::from_slice(self.extrinsic_hash.as_ref())
//                 )?;
//                 write!(f, "index: {}\n", self.index)?;
//                 write!(
//                     f,
//                     "withdraw {:?} <- {}\n",
//                     H256::from(&self.who),
//                     self.amount
//                 )
//             }
//         }
 
//     }

//     use std::fmt::Debug;

//     use serde::de::DeserializeOwned;
//     use serde::Deserialize;
//     use serde::Serialize;
//     use subxt::utils::H256;

//     use super::runtime;

//     #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
//     pub enum BalanceEvent {
//         transfer {
//             from: [u8; 32],
//             to: [u8; 32],
//             amount: u128,
//         },
//     }

//     #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
//     pub struct Balance {
//         pub block_hash: Vec<u8>,
//         pub block_number: u64,
//         pub block_time: u64, // TODO: not used currently.
//         pub extrinsic_hash: Vec<u8>,
//         pub index: u32,
//         pub call: BalanceEvent,
//     }

//     impl std::fmt::Display for Balance {
//         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             write!(f, "Balance\n")?;
//             write!(f, "  block_number: {}\n", self.block_number)?;
//             write!(
//                 f,
//                 "  block_hash: {:?}\n",
//                 H256::from_slice(self.block_hash.as_ref())
//             )?;
//             write!(
//                 f,
//                 "  extrinsic_hash: {:?}\n",
//                 H256::from_slice(self.extrinsic_hash.as_ref())
//             )?;
//             write!(f, "  index: {}\n", self.index)?;
//             match self.call {
//                 BalanceEvent::transfer { from, to, amount } => {
//                     write!(f, "  Transfer\n")?;
//                     write!(
//                         f,
//                         "    {:?} => {:?}, {}\n",
//                         H256::from(&from),
//                         H256::from(&to),
//                         amount
//                     )?;
//                 }
//             }
//             write!(f, "\n")

//             // match
//         }
//     }
// }

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
