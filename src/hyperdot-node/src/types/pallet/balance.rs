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

use crate::types::pallet::utils::AccountId32;

// pub enum Event {
//     #[doc = "An account was created with some free balance."]
//     Endowed {
//         account: AccountId32,
//         free_balance: u128,
//     },
//     #[doc = "An account was removed whose balance was non-zero but below ExistentialDeposit,"]
//     #[doc = "resulting in an outright loss."]
//     DustLost { account: AccountId32, amount: u128 },
//     #[doc = "Transfer succeeded."]
//     Transfer {
//         from: AccountId32,
//         to: AccountId32,
//         amount: u128,
//     },
//     #[doc = "A balance was set by root."]
//     BalanceSet { who: AccountId32, free: u128 },
//     #[doc = "Some balance was reserved (moved from free to reserved)."]
//     Reserved { who: AccountId32, amount: u128 },
//     #[doc = "Some balance was unreserved (moved from reserved to free)."]
//     Unreserved { who: AccountId32, amount: u128 },
//     #[doc = "Some balance was moved from the reserve of the first account to the second account."]
//     #[doc = "Final argument indicates the destination balance type."]
//     ReserveRepatriated {
//         from: AccountId32,
//         to: AccountId32,
//         amount: u128,
//         destination_status: runtime_types::frame_support::traits::tokens::misc::BalanceStatus,
//     },
//     #[doc = "Some amount was deposited (e.g. for transaction fees)."]
//     Deposit { who: AccountId32, amount: u128 },
//     #[doc = "Some amount was withdrawn from the account (e.g. for transaction fees)."]
//     Withdraw { who: AccountId32, amount: u128 },
//     #[doc = "Some amount was removed from the account (e.g. for misbehavior)."]
//     Slashed { who: AccountId32, amount: u128 },
//     #[doc = "Some amount was minted into an account."]
//     Minted { who: AccountId32, amount: u128 },
//     #[doc = "Some amount was burned from an account."]
//     Burned { who: AccountId32, amount: u128 },
//     #[doc = "Some amount was suspended from an account (it can be restored later)."]
//     Suspended { who: AccountId32, amount: u128 },
//     #[doc = "Some amount was restored into an account."]
//     Restored { who: AccountId32, amount: u128 },
//     #[doc = "An account was upgraded."]
//     Upgraded { who: AccountId32 },
//     #[doc = "Total issuance was increased by `amount`, creating a credit to be balanced."]
//     Issued { amount: u128 },
//     #[doc = "Total issuance was decreased by `amount`, creating a debt to be balanced."]
//     Rescinded { amount: u128 },
//     #[doc = "Some balance was locked."]
//     Locked { who: AccountId32, amount: u128 },
//     #[doc = "Some balance was unlocked."]
//     Unlocked { who: AccountId32, amount: u128 },
//     #[doc = "Some balance was frozen."]
//     Frozen { who: AccountId32, amount: u128 },
//     #[doc = "Some balance was thawed."]
//     Thawed { who: AccountId32, amount: u128 },
// }

pub mod event {
    use std::fmt::Debug;

    use serde::de::DeserializeOwned;
    use serde::Deserialize;
    use serde::Serialize;
    use subxt::utils::H256;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct Transfer {
        pub from: [u8; 32],
        pub to: [u8; 32],
        pub amount: u128,
        pub success: bool, // streaming?
    }

    impl std::fmt::Display for Transfer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // write!(f, "block_number: {}\n", self.block_number)?;
            // write!(
            //     f,
            //     "block_hash: {:?}\n",
            //     H256::from_slice(self.block_hash.as_ref())
            // )?;
            // write!(
            //     f,
            //     "extrinsic_hash: {:?}\n",
            //     H256::from_slice(self.extrinsic_hash.as_ref())
            // )?;
            // write!(f, "index: {}\n", self.index)?;
            write!(
                f,
                "transfer {:?} => {:?}, {}\n",
                H256::from(&self.from),
                H256::from(&self.to),
                self.amount
            )
        }
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct Withdraw {
        pub who: [u8; 32],
        pub amount: u128,
        pub success: bool,
    }

    impl std::fmt::Display for Withdraw {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // write!(f, "block_number: {}\n", self.block_number)?;
            // write!(
            //     f,
            //     "block_hash: {:?}\n",
            //     H256::from_slice(self.block_hash.as_ref())
            // )?;
            // write!(
            //     f,
            //     "extrinsic_hash: {:?}\n",
            //     H256::from_slice(self.extrinsic_hash.as_ref())
            // )?;
            // write!(f, "index: {}\n", self.index)?;
            write!(
                f,
                "withdraw {:?} <- {}\n",
                H256::from(&self.who),
                self.amount
            )
        }
    }
}
