use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use hyperdot_core::config::Catalog;

use super::streaming::BlockStreaming2;
use super::streaming::BlockStreamingHandle2;
use crate::streaming::speaker;

struct ChainStreamingState {
    streming_handle: BlockStreamingHandle2,
}

pub struct StreamingController {
    catalog: Catalog,
    speaker_controller: Arc<speaker::Controller>,
    chains: HashMap<String, ChainStreamingState>,
}

impl StreamingController {
    pub async fn async_new(catalog: Catalog) -> anyhow::Result<Self> {
        let speaker_controller = speaker::Controller::async_new(catalog.clone()).await?;
        Ok(Self {
            catalog,
            speaker_controller: Arc::new(speaker_controller),
            chains: HashMap::new(),
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        for chain in self.catalog.chain.iter() {
            if !chain.enabled {
                tracing::info!("💁 {}: chain not enabled, skippd", chain.name);
                continue;
            }

            let storage_node_names = chain.storage_nodes.as_ref().map_or(
                Err(anyhow!(
                    "🛕 {}: chain is enabled but storage nodes empty",
                    chain.name
                )),
                |sn| Ok(sn),
            )?;

            let mut storage_nodes = vec![];
            for storage_node_name in storage_node_names.iter() {
                storage_nodes.push(
                    self.catalog
                        .storage
                        .nodes
                        .iter()
                        .find(|node| node.name == *storage_node_name)
                        .map_or(
                            Err(anyhow!(
                                "👷‍♀️ {}: chain expected storage node {} not find",
                                chain.name,
                                storage_node_name
                            )),
                            |node| Ok(node.clone()),
                        )?,
                );
            }

            tracing::info!("🥳 {}: good catalog, start streaming", chain.name);
            let streming_handle =
                BlockStreaming2::spawn(chain, &storage_nodes, self.speaker_controller.clone())
                    .await?;
            self.chains
                .insert(chain.name.clone(), ChainStreamingState { streming_handle });
        }

        Ok(())
    }

    pub async fn stopped(self) -> anyhow::Result<()> {
        for (_, state) in self.chains.into_iter() {
            state.streming_handle.stopped().await?;
        }
        Ok(())
    }
}
