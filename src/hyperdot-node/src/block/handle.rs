use std::marker::PhantomData;

use subxt::client::OfflineClient;
use subxt::client::OfflineClientT;
use subxt::config::Header;
use subxt::Config;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

use super::sync::CachedBody;
use crate::runtime_api::polkadot;
use crate::types::pallet::system::support;
use crate::types::BlockDescribe;
use crate::types::BlockHeaderDescribe;
use crate::types::ExtrinsicDescribe;

pub const UNKOWN_PALLET_NAME: &'static str = "unkown_pallet";

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
}

impl<T, C> BlockHandleImpl<T, C>
where
    T: Config,
    C: OfflineClientT<T>,
{
    /// Create a block handle.
    pub fn new(header: T::Header, body: CachedBody<T, C>) -> Self {
        Self { header, body }
    }
}

impl BlockHandler<PolkadotConfig, OnlineClient<PolkadotConfig>>
    for BlockHandleImpl<PolkadotConfig, OnlineClient<PolkadotConfig>>
{
    type Output = BlockDescribe;

    fn handle(self) -> anyhow::Result<Self::Output> {
        let block_hash = self.header.hash();
        let block_number = self.header.number();
        let parent_hash = self.header.parent_hash;
        let state_root = self.header.state_root;
        let extrinsics_root = self.header.extrinsics_root;

        let block_header_desc = BlockHeaderDescribe {
            block_number: block_number as u64,
            block_hash: block_hash.as_bytes().to_vec(),
            parent_hash: parent_hash.as_bytes().to_vec(),
            state_root: state_root.as_bytes().to_vec(),
            extrinsics_root: extrinsics_root.as_bytes().to_vec(),
        };

        let mut block_extrinsics_desc = vec![];
        for (i, ext) in self.body.details.iter().enumerate() {
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
                                crate::types::ExtrinsicEventDescribe::Transfer(
                                    crate::types::pallet::balance::event::Transfer {
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
                                crate::types::ExtrinsicEventDescribe::Withdraw(
                                    crate::types::pallet::balance::event::Withdraw {
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

            let extrinsic_index = ext.index();
            let extrinsic_pallet_index = ext.pallet_index();
            let extrinsic_pallet_name = ext
                .pallet_name()
                .map_or(UNKOWN_PALLET_NAME.to_string(), |name| name.to_string());
            let extrinsic_hash = events.extrinsic_hash();
            block_extrinsics_desc.push(ExtrinsicDescribe {
                index: extrinsic_index,
                pallet_index: extrinsic_pallet_index,
                pallet_name: extrinsic_pallet_name,
                hash: extrinsic_hash.as_bytes().to_vec(),
                events: writable_extrinsic_events,
            })
        }

        Ok(BlockDescribe {
            header: block_header_desc,
            extrinsics: block_extrinsics_desc,
        })
    }
}
