use std::marker::PhantomData;

use subxt::events::RootEvent;
use subxt::events::StaticEvent;

use crate::runtime_api::polkadot;
use crate::types::RawEvent;

/// The trait hanlde pallet event.
pub trait PalletEventHandler<E>
where E: RootEvent
{
    type OutputRaw;

    type OutputIndexed;

    fn handle(&mut self, root_event: &E) -> anyhow::Result<()>;

    fn get_raw(&self) -> Option<Self::OutputRaw>;

    fn get_indexed(&self) -> Option<Self::OutputIndexed>;
}

/// PalletEventHandler impl for mulitple chain.
pub struct PalletEventHandle<E>
where E: RootEvent
{
    naming: Option<(String, String)>,
    _m: PhantomData<E>,
}

impl<E> PalletEventHandle<E>
where E: RootEvent
{
    pub fn new() -> Self {
        Self {
            naming: None,
            _m: PhantomData,
        }
    }
}

impl PalletEventHandle<polkadot::Event> {
    fn system(&mut self, e: &polkadot::system::Event) -> anyhow::Result<()> {
        let event_name = match e {
            polkadot::system::Event::ExtrinsicSuccess { .. } => "ExtrinsicSuccess",
            polkadot::system::Event::ExtrinsicFailed { .. } => "ExtrinsicFailed",
            polkadot::system::Event::CodeUpdated { .. } => "CodeUpdated",
            polkadot::system::Event::NewAccount { .. } => "NewAccount",
            polkadot::system::Event::KilledAccount { .. } => "KilledAccount",
            polkadot::system::Event::Remarked { .. } => "Remarked",
            _ => "Unkown",
        };

        self.naming = Some(("System".to_string(), event_name.to_string()));

        Ok(())
    }

    fn indices(&mut self, e: &polkadot::indices::Event) -> anyhow::Result<()> {
        let event_name = match e {
            polkadot::indices::Event::IndexAssigned { .. } => "IndexAssigned",
            polkadot::indices::Event::IndexFreed { .. } => "IndexFreed",
            polkadot::indices::Event::IndexFrozen { .. } => "IndexFrozen",
            _ => "Unkown",
        };

        // serde_json::to_string(e).unwrap();

        self.naming = Some(("System".to_string(), event_name.to_string()));

        Ok(())
    }
}

impl PalletEventHandler<polkadot::Event> for PalletEventHandle<polkadot::Event> {
    type OutputRaw = (String /* pallet */, String /* Event */);

    type OutputIndexed = ();

    fn handle(&mut self, root_event: &polkadot::Event) -> anyhow::Result<()> {
        match root_event {
            polkadot::Event::System(e) => self.system(e),
            _ => todo!(),
        }
    }

    fn get_raw(&self) -> Option<Self::OutputRaw> {
        self.naming.clone()
    }

    fn get_indexed(&self) -> Option<Self::OutputIndexed> {
        todo!()
    }
}
