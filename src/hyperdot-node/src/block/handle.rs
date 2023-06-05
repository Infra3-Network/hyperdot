use std::marker::PhantomData;

use subxt::blocks::ExtrinsicDetails;
use subxt::client::OfflineClient;
use subxt::client::OfflineClientT;
use subxt::config::Header;
use subxt::Config;
use subxt::PolkadotConfig;

use super::CachedBody;
use crate::runtime_api::polkadot;
use crate::types::WritableExtrinsic;
use crate::types::pallet::balance;
use crate::types::pallet::system::support;
use crate::types::WritableBlock;
use crate::types::WritableBlockHeader;

pub trait BlockHandler<T, C>
where
    T: Config,
    C: OfflineClientT<T>,
{
    type Output;
    fn handle(self) -> anyhow::Result<Self::Output>;
}

pub struct BlockHandleImpl<T, C>
where
    T: Config,
    C: OfflineClientT<T>,
{
    header: T::Header,
    body: CachedBody<T, C>,

    output: WritableBlock,
    _m1: PhantomData<T>,
    _m2: PhantomData<C>,
}

impl BlockHandler<PolkadotConfig, OfflineClient<PolkadotConfig>>
    for BlockHandleImpl<PolkadotConfig, OfflineClient<PolkadotConfig>>
{
    type Output = WritableBlock;

    fn handle(self) -> anyhow::Result<Self::Output> {
        let block_hash = self.header.hash();
        let block_number = self.header.number();
        let parent_hash = self.header.parent_hash;
        let state_root = self.header.state_root;
        let extrinsics_root = self.header.extrinsics_root;

        let writable_header = WritableBlockHeader {
            block_number: block_number as u64,
            block_hash: block_hash.as_bytes().to_vec(),
            parent_hash: parent_hash.as_bytes().to_vec(),
            state_root: state_root.as_bytes().to_vec(),
            extrinsics_root: extrinsics_root.as_bytes().to_vec(),
        };

        let mut writable_extrinsics = vec![];
        for (i, ext) in self.body.details.iter().enumerate() {
            let extrinsic_index = ext.index();

            let events = &self.body.events[i];
            let mut writable_extrinsic_events = vec![];

            // find system event if success or failed
            let mut extrinsic_success = None;
            for event in events.iter() {
                let event = event?;
                if let Some(success) =
                    event.as_event::<polkadot::system::events::ExtrinsicSuccess>()?
                {
                    let mut writable_success =
                        crate::types::pallet::system::event::ExtrinsicSuccess::default();

                    match success.dispatch_info.class {
                        polkadot::runtime_types::frame_support::dispatch::DispatchClass::Normal => {
                            writable_success.dispatch_info.class = support::DispatchClass::Normal;
                        },
                        polkadot::runtime_types::frame_support::dispatch::DispatchClass::Operational => {
                            writable_success.dispatch_info.class = support::DispatchClass::Operational;
                        },
                        polkadot::runtime_types::frame_support::dispatch::DispatchClass::Mandatory => {
                            writable_success.dispatch_info.class = support::DispatchClass::Operational;
                        },
                    }

                    match success.dispatch_info.pays_fee {
                        polkadot::runtime_types::frame_support::dispatch::Pays::Yes => {
                            writable_success.dispatch_info.pays_fee = support::Pays::Yes;
                        }
                        polkadot::runtime_types::frame_support::dispatch::Pays::No => {
                            writable_success.dispatch_info.pays_fee = support::Pays::No;
                        }
                    }

                    writable_success.dispatch_info.weight.proof_size =
                        success.dispatch_info.weight.proof_size;
                    writable_success.dispatch_info.weight.ref_time =
                        success.dispatch_info.weight.ref_time;
                    extrinsic_success = Some(writable_success);
                }

                if let Some(failed) =
                    event.as_event::<polkadot::system::events::ExtrinsicSuccess>()?
                {
                    // TODO: impl
                }
            }

            for event in events.iter() {
                let event = event?;
                let root_event = event.as_root_event::<polkadot::Event>()?;
                match root_event {
                    polkadot::Event::Balances(balance_event) => match balance_event {
                        polkadot::balances::Event::Transfer { from, to, amount } => {
                            writable_extrinsic_events.push(
                                crate::types::WritableExtrinsicEvent::Transfer(
                                    crate::types::pallet::balance::event::Transfer {
                                        index: extrinsic_index,
                                        from: from.0,
                                        to: to.0,
                                        amount,
                                        success: if extrinsic_success.is_some() {
                                            true
                                        } else {
                                            false
                                        }, // TODO streaming with final event
                                    },
                                ),
                            );
                        }
                        polkadot::balances::Event::Withdraw { who, amount } => {
                            writable_extrinsic_events.push(
                                crate::types::WritableExtrinsicEvent::Withdraw(
                                    crate::types::pallet::balance::event::Withdraw {
                                        index: extrinsic_index,
                                        who: who.0,
                                        amount: amount,
                                        success: if extrinsic_success.is_some() {
                                            true
                                        } else {
                                            false
                                        },
                                    },
                                ),
                            );
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            
            writable_extrinsics.push(WritableExtrinsic{
                events: writable_extrinsic_events,
            })
        }
        todo!()
    }
}

// impl<T, C> BlockHandle<T, C>
// where
//     T: Config,
//     C: OfflineClientT<T>,
// {
//     pub fn handle(&mut self) {
//         let block_hash = self.header.hash();
//         let block_number = self.header.number().into();
//         self.header.

//         let writable_header = WritableBlockHeader {
//             block_number,
//             block_hash,
//             parent_hash,
//             state_root,
//             extrinsics_root,
//         };
//     }

// }
