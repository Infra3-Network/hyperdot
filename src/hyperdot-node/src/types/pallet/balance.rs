
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


pub mod event {
    use std::fmt::Debug;

    use serde::de::DeserializeOwned;
    use serde::Deserialize;
    use serde::Serialize;
    use subxt::utils::H256;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct Transfer {
        // pub block_hash: Vec<u8>,
        // pub block_number: u64,
        // pub block_time: u64, // TODO: not used currently.
        // pub extrinsic_hash: Vec<u8>,
        // pub index: u32,
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
        // pub block_hash: Vec<u8>,
        // pub block_number: u64,
        // pub block_time: u64, // TODO: not used currently.
        // pub extrinsic_hash: Vec<u8>,
        // pub index: u32,
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
