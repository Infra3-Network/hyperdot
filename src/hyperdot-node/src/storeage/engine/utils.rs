use subxt::events::RootEvent;

/// Hold five topics for multi-topic in event.
pub struct FiveTopics {
    pub t0: String,
    pub t1: String,
    pub t2: String,
    pub t3: String,
    pub t4: String,
}

impl From<&Vec<Vec<u8>>> for FiveTopics {
    fn from(topics: &Vec<Vec<u8>>) -> Self {
        let mut result = FiveTopics {
            t0: String::new(),
            t1: String::new(),
            t2: String::new(),
            t3: String::new(),
            t4: String::new(),
        };
        match topics.len() {
            0 => {}
            1 => {
                result.t0 = format!("0x{}", hex::encode(&topics[0]));
            }
            2 => {
                result.t0 = format!("0x{}", hex::encode(&topics[0]));
                result.t1 = format!("0x{}", hex::encode(&topics[1]));
            }
            3 => {
                result.t0 = format!("0x{}", hex::encode(&topics[0]));
                result.t1 = format!("0x{}", hex::encode(&topics[1]));
                result.t2 = format!("0x{}", hex::encode(&topics[2]));
            }

            4 => {
                result.t0 = format!("0x{}", hex::encode(&topics[0]));
                result.t1 = format!("0x{}", hex::encode(&topics[1]));
                result.t2 = format!("0x{}", hex::encode(&topics[2]));
                result.t3 = format!("0x{}", hex::encode(&topics[3]));
            }

            5 | _ => {
                result.t0 = format!("0x{}", hex::encode(&topics[0]));
                result.t1 = format!("0x{}", hex::encode(&topics[1]));
                result.t2 = format!("0x{}", hex::encode(&topics[2]));
                result.t3 = format!("0x{}", hex::encode(&topics[3]));
                result.t4 = format!("0x{}", hex::encode(&topics[4]));
            }
        }

        result
    }
}

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::runtime_api::polkadot;
use crate::runtime_api::GetName;

struct RuntimeEventHandle<E>
where E: RootEvent + GetName
{
    callbacks: HashMap<String, Box<dyn Fn(E) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>>>>,
}

impl<E> Default for RuntimeEventHandle<E>
where E: RootEvent + GetName
{
    fn default() -> Self {
        Self {
            callbacks: Default::default(),
        }
    }
}

impl<E> RuntimeEventHandle<E>
where E: RootEvent + GetName
{
    fn register<F>(&mut self, callback_name: &str, callback: F)
    where F: Fn(E) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>> + 'static {
        todo!()
    }

    async fn handle(&mut self, event: E) -> anyhow::Result<()> {
        let (pallet_name, name) = event.name();
        if !self.callbacks.contains_key(&name) {
            return Ok(());
        }

        let handle = self.callbacks.get_mut(&name).unwrap();
        handle(event).await
    }
}
