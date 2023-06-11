use subxt::events::RootEvent;

/// Hold five topics for multi-topic in event.
pub struct FiveTopics {
    pub t0: Vec<u8>,
    pub t1: Vec<u8>,
    pub t2: Vec<u8>,
    pub t3: Vec<u8>,
    pub t4: Vec<u8>,
}

impl From<&Vec<Vec<u8>>> for FiveTopics {
    fn from(topics: &Vec<Vec<u8>>) -> Self {
        let mut result = FiveTopics {
            t0: vec![],
            t1: vec![],
            t2: vec![],
            t3: vec![],
            t4: vec![],
        };
        match topics.len() {
            0 => {}
            1 => {
                result.t0 = topics[0].clone();
            }
            2 => {
                result.t0 = topics[0].clone();
                result.t1 = topics[1].clone();
            }
            3 => {
                result.t0 = topics[0].clone();
                result.t1 = topics[1].clone();
                result.t2 = topics[2].clone();
            }

            4 => {
                result.t0 = topics[0].clone();
                result.t1 = topics[1].clone();
                result.t2 = topics[2].clone();
                result.t3 = topics[3].clone();
            }

            5 | _ => {
                result.t0 = topics[0].clone();
                result.t1 = topics[1].clone();
                result.t2 = topics[2].clone();
                result.t3 = topics[3].clone();
                result.t4 = topics[4].clone();
            }
        }

        result
    }
}

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::runtime_api::polkadot;
use crate::runtime_api::Naming;

struct RuntimeEventHandle<E>
where E: RootEvent + Naming
{
    callbacks: HashMap<String, Box<dyn Fn(E) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>>>>,
}

impl<E> Default for RuntimeEventHandle<E>
where E: RootEvent + Naming
{
    fn default() -> Self {
        Self {
            callbacks: Default::default(),
        }
    }
}

impl<E> RuntimeEventHandle<E>
where E: RootEvent + Naming
{
    fn register<F>(&mut self, callback_name: &str, callback: F)
    where F: Fn(E) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>> + 'static {
        todo!()
    }

    async fn handle(&mut self, event: E) -> anyhow::Result<()>{
        let name = event.what();
        if !self.callbacks.contains_key(&name) {
            return Ok(())
        }

        let handle = self.callbacks.get_mut(&name).unwrap();
        handle(event).await
    }
}

#[test]
fn test_runtime_event_peek() {
    let ev_peek = RuntimeEventHandle::<polkadot::Event>::default();
    ev_peek.register("test", |e| {
        let fut = async move {
            todo!();
        };

        Box::pin(fut)
    });
}
