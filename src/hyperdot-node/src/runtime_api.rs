// use std::env;
// use std::path::Path;
// lazy_static::lazy_static! {
//     static ref POLKADOT_METADATA_SMALL_PATH: String = {
//         let manifest_dir =
//             env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable unset");
//         let metadatas_dir = Path::new(&manifest_dir).join("metadatas").join("polkadot_metadata_small.scale");
//         metadatas_dir.as_os_str().to_str().unwrap().to_string()
//     };
// }

#[subxt::subxt(runtime_metadata_path = "../../metadatas/polkadot_metadata_full.scale")]
pub mod polkadot {}

/// GetName return pallet name and call or event name.
pub trait GetName {
    fn name(&self) -> (String, String);
}

impl GetName for polkadot::Event {
    fn name(&self) -> (String, String) {
        let unsupport_event_name = "Unsupport";
        match self {
            polkadot::Event::System(system) => {
                let pallet = "system";
                match system {
                    polkadot::system::Event::ExtrinsicSuccess { .. } => {
                        (pallet.to_string(), "ExtrinsicSuccess".to_string())
                    }

                    polkadot::system::Event::ExtrinsicFailed { .. } => {
                        (pallet.to_string(), "ExtrinsicFailed".to_string())
                    }

                    polkadot::system::Event::CodeUpdated => {
                        (pallet.to_string(), "CodeUpdated".to_string())
                    }

                    polkadot::system::Event::NewAccount { .. } => {
                        (pallet.to_string(), "NewAccount".to_string())
                    }

                    polkadot::system::Event::KilledAccount { .. } => {
                        (pallet.to_string(), "KilledAccount".to_string())
                    }

                    polkadot::system::Event::Remarked { .. } => {
                        (pallet.to_string(), "Remarked".to_string())
                    }

                    _ => (pallet.to_string(), unsupport_event_name.to_string()),
                }
            }

            polkadot::Event::Indices(indices) => {
                let pallet = "Indices";
                todo!()
            }

            polkadot::Event::Balances(balance) => {
                let pallet = "balances";
                match balance {
                    polkadot::balances::Event::Endowed { .. } => {
                        (pallet.to_string(), "Endowed".to_string())
                    }

                    polkadot::balances::Event::DustLost { .. } => {
                        (pallet.to_string(), "DustLost".to_string())
                    }

                    polkadot::balances::Event::Transfer { .. } => {
                        (pallet.to_string(), "Transfer".to_string())
                    }

                    polkadot::balances::Event::BalanceSet { .. } => {
                        (pallet.to_string(), "BalanceSet".to_string())
                    }

                    polkadot::balances::Event::Reserved { .. } => {
                        (pallet.to_string(), "Reserved".to_string())
                    }

                    polkadot::balances::Event::Unreserved { .. } => {
                        (pallet.to_string(), "Unreserved".to_string())
                    }

                    polkadot::balances::Event::ReserveRepatriated { .. } => {
                        (pallet.to_string(), "ReserveRepatriated".to_string())
                    }

                    polkadot::balances::Event::Deposit { .. } => {
                        (pallet.to_string(), "Deposit".to_string())
                    }

                    polkadot::balances::Event::Withdraw { .. } => {
                        (pallet.to_string(), "Withdraw".to_string())
                    }

                    polkadot::balances::Event::Slashed { .. } => {
                        (pallet.to_string(), "Slashed".to_string())
                    }

                    polkadot::balances::Event::Minted { .. } => {
                        (pallet.to_string(), "Minted".to_string())
                    }

                    polkadot::balances::Event::Burned { .. } => {
                        (pallet.to_string(), "Burned".to_string())
                    }

                    polkadot::balances::Event::Restored { .. } => {
                        (pallet.to_string(), "Restored".to_string())
                    }

                    polkadot::balances::Event::Upgraded { .. } => {
                        (pallet.to_string(), "ReserveRepatriated".to_string())
                    }

                    polkadot::balances::Event::Issued { .. } => {
                        (pallet.to_string(), "Issued".to_string())
                    }

                    polkadot::balances::Event::Rescinded { .. } => {
                        (pallet.to_string(), "Rescinded".to_string())
                    }

                    polkadot::balances::Event::Locked { .. } => {
                        (pallet.to_string(), "Locked".to_string())
                    }

                    polkadot::balances::Event::Unlocked { .. } => {
                        (pallet.to_string(), "Unlocked".to_string())
                    }

                    polkadot::balances::Event::Frozen { .. } => {
                        (pallet.to_string(), "Frozen".to_string())
                    }

                    polkadot::balances::Event::Thawed { .. } => {
                        (pallet.to_string(), "Thawed".to_string())
                    }

                    _ => (pallet.to_string(), unsupport_event_name.to_string()),
                }
            }

            polkadot::Event::TransactionPayment(txp) => {
                let pallet = "transaction_payment";
                match txp {
                    polkadot::transaction_payment::Event::TransactionFeePaid { .. } => {
                        (pallet.to_string(), "TransactionFeePaid".to_string())
                    }

                    _ => (pallet.to_string(), unsupport_event_name.to_string()),
                }
            }

            polkadot::Event::Utility(utility) => {
                let pallet = "utility";
                match utility {
                    polkadot::utility::Event::BatchInterrupted { .. } => {
                        (pallet.to_string(), "BatchInterrupted".to_string())
                    }

                    polkadot::utility::Event::BatchCompleted => {
                        (pallet.to_string(), "BatchCompleted".to_string())
                    }

                    polkadot::utility::Event::BatchCompletedWithErrors { .. } => {
                        (pallet.to_string(), "BatchCompletedWithErrors".to_string())
                    }

                    polkadot::utility::Event::ItemCompleted => {
                        (pallet.to_string(), "ItemCompleted".to_string())
                    }

                    polkadot::utility::Event::ItemFailed { .. } => {
                        (pallet.to_string(), "ItemFailed".to_string())
                    }

                    polkadot::utility::Event::DispatchedAs { .. } => {
                        (pallet.to_string(), "DispatchedAs".to_string())
                    }

                    _ => (pallet.to_string(), unsupport_event_name.to_string()),
                }
            }
            _ => unimplemented!(),
        }
    }
}
