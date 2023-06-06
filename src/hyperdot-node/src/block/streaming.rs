use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::speaker::SpeakerController;
use crate::types::BlockDescribe;

// pub trait StreamFilter {
//     type Data;

//     fn filter(data: Self::Data) -> anyhow::Result<Self::Data>;
// }

pub struct Streaming {
    speaker: Arc<SpeakerController>,
}

impl Streaming {
    pub fn new() -> Self {
        let speaker = SpeakerController::new();
        Self {
            speaker: Arc::new(speaker),
        }
    }
}

impl Streaming {
    pub fn start(&mut self, mut rx: UnboundedReceiver<BlockDescribe>) {
        tokio::spawn(async move {
            loop {
                let block_desc = match rx.recv().await {
                    None => {
                        tracing::error!("block channel closed");
                        return;
                    }
                    Some(block_desc) => block_desc,
                };

                println!("{}", serde_json::to_string_pretty(&block_desc).unwrap());
            }
        });
    }
}
