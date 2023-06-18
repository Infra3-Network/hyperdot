use std::collections::HashMap;
use std::sync::Arc;

use hyperdot_common_config::Catalog;
use tokio::sync::RwLock;

use super::child::JsonRpcChild;
use super::child::SpeakerJsonRpcChild;
use crate::types::rpc::WriteBlock;
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
        T: Clone + Send + serde::Serialize,
    {
        let rl = self.childs.read().await;
        for child in rl.iter() {
            child.write_block(request.clone()).await?;
        }

        Ok(WriteBlockResponse {})
    }
}

pub struct Controller {
    // childs: Arc<RwLock<Vec<SpeakerJsonRpcChild>>>,
    multi_chain: RwLock<HashMap<String, Vec<Arc<JsonRpcChild>>>>,
}

impl Controller {
    pub async fn async_new(catalog: Catalog) -> anyhow::Result<Self> {
        // let childs = super::url::parse_childs(urls).await?;
        // Ok(Self {
        //     childs: Arc::new(RwLock::new(childs)),
        // })
        let mut multi_chain = HashMap::new();
        for chain in catalog.chain.iter() {
            if (!chain.enabled) {
                tracing::info!("💁 {}: skipped not enabled", chain.name);
                continue;
            }

            let snodes = match chain.storage_nodes.as_ref() {
                None => {
                    tracing::info!("💁 {}: skipped not define storage_nodes", chain.name);
                    continue;
                }
                Some(nodes) => {
                    if nodes.is_empty() {
                        tracing::info!("💁 {}: skipped defined storage_nodes empty", chain.name);
                        continue;
                    } else {
                        nodes.clone()
                    }
                }
            };

            let mut snode_cfg = vec![];
            let mut snode_matchs = vec![];
            let mut snode_not_matchs = vec![];

            for snode in snodes.iter() {
                match catalog
                    .storage
                    .nodes
                    .iter()
                    .find(|node| node.name == *snode)
                {
                    None => {
                        snode_not_matchs.push(snode);
                        continue;
                    }
                    Some(node_cfg) => {
                        snode_cfg.push(node_cfg.clone());
                        snode_matchs.push(snode)
                    }
                }
            }

            if snode_cfg.is_empty() {
                tracing::warn!(
                    "💁 {}: skipped empty storage.nodes empty that not match for the chain defined storage_nodes({:?})",
                    chain.name,
                    snodes,
                );
                continue;
            }

            if snode_cfg.len() != snodes.len() {
                tracing::warn!(
                    "🌦️ {}: storage_nodes({:?}) defined by storage.nodes and chain does not match, matches({:?}), not matches({:?})",
                    chain.name,
                    snodes,
                    snode_matchs,
                    snode_not_matchs
                );
            }

            let mut chian_jsonrpc_childs = vec![];
            let mut not_available_childs = vec![];
            for snode_cfg in snode_cfg.iter() {
                match JsonRpcChild::open(snode_cfg).await {
                    Err(err) => {
                        tracing::error!(
                            "💁 {}: storage node({}) connect json-rpc server error: {}",
                            chain.name,
                            snode_cfg.name,
                            err
                        );
                        continue;
                    }
                    Ok(child) => {
                        chian_jsonrpc_childs.push(Arc::new(child));
                        not_available_childs.push(snode_cfg.name.clone());
                    }
                }
            }
            if chian_jsonrpc_childs.is_empty() {
                tracing::error!(
                    "💁 {}: speaker cannot initialized, all({:?}) storage_nodes json-rpc server not available",
                    chain.name,
                    not_available_childs,
                );
            }

            multi_chain.insert(chain.name.clone(), chian_jsonrpc_childs);
        }

        return Ok(Self {
            multi_chain: RwLock::new(multi_chain),
        });
    }

    pub async fn write_block(&self, request: WriteBlock) -> anyhow::Result<WriteBlockResponse> {
        let chain_name = &request.chain.name;
        let childs = {
            let rl = self.multi_chain.read().await;
            if !rl.contains_key(chain_name) {
                return Err(anyhow::anyhow!(
                    "{}: no available storage node exists in the chain",
                    chain_name
                ));
            }
            rl.get(chain_name).unwrap().clone()
        };

        for child in childs.iter() {
            child.write_block(request.clone()).await?;
        }

        Ok(WriteBlockResponse {})
    }
}
