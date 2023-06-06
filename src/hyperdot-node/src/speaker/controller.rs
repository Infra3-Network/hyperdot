use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::RwLock;

use super::SpeakerChild;
use super::SpeakerOps;
use crate::types::WriteBlockRequest;
use crate::types::WriteBlockResponse;

pub struct SpeakerChildHandle {}

pub struct SpeakerController {
    childs: Arc<RwLock<Vec<Box<dyn SpeakerChild>>>>,
}

impl SpeakerController {
    pub fn new() -> Self {
        Self {
            childs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add child into controller. `None` returned if given name exists controller.
    pub async fn add_child(
        &self,
        name: &str,
        child: Box<dyn SpeakerChild>,
    ) -> Option<Box<dyn SpeakerChild>> {
        {
            let rl = self.childs.read().await;
            if rl.iter().find(|c| c.name().as_str() == name).is_some() {
                return Some(child);
            }
        }

        let mut wl = self.childs.write().await;

        wl.push(child);
        None
    }

    /// remove child into controller. `None` returned if given name associated child not exists controller.
    pub async fn remove_child(&self, name: &str) -> Option<Box<dyn SpeakerChild>> {
        let index = {
            let rl = self.childs.read().await;
            match rl.iter().position(|c| c.name().as_str() == name) {
                None => return None,
                Some(index) => index,
            }
        };

        let mut wl = self.childs.write().await;
        Some(wl.swap_remove(index))
    }
}

#[async_trait::async_trait]
impl SpeakerOps for SpeakerController {
    async fn write_block(&self, request: WriteBlockRequest) -> anyhow::Result<WriteBlockResponse> {
        let rl = self.childs.read().await;
        for child in rl.iter() {
            child.write_block(request.clone()).await?;
        }

        Ok(WriteBlockResponse {})
    }
}
