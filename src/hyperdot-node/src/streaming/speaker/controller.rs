use std::sync::Arc;


use tokio::sync::RwLock;

use super::child::SpeakerJsonRpcChild;

// use super::SpeakerChild;
// use super::SpeakerOps;
use crate::types::rpc::WriteBlockRequest;
use crate::types::rpc::WriteBlockResponse;

pub struct SpeakerChildHandle {}

pub struct SpeakerController {
    childs: Arc<RwLock<Vec<SpeakerJsonRpcChild>>>,
}

impl SpeakerController {
    pub async fn new(urls: &[String]) -> anyhow::Result<Self> {
        let childs = super::url::parse_childs(urls).await?;
        Ok(Self {
            childs: Arc::new(RwLock::new(childs)),
        })
    }

    /// Add child into controller. `None` returned if given name exists controller.
    pub async fn add_cild(
        &self,
        name: &str,
        child: SpeakerJsonRpcChild,
    ) -> Option<SpeakerJsonRpcChild> {
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
    pub async fn remove_child(&self, name: &str) -> Option<SpeakerJsonRpcChild> {
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

    pub async fn write_block<T>(
        &self,
        request: WriteBlockRequest<T>,
    ) -> anyhow::Result<WriteBlockResponse> 
    where
        T: Clone + Send + serde::Serialize
    {
        let rl = self.childs.read().await;
        for child in rl.iter() {
            child.write_block(request.clone()).await?;
        }

        Ok(WriteBlockResponse {})
    }
}
