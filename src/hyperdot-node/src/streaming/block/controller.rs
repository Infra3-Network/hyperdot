use anyhow::anyhow;
use hyperdot_common_config::Catalog;

use super::streaming::BlockStreaming2;

pub struct StreamingController {
    catalog: Catalog,
}

impl StreamingController {
    pub fn new(catalog: Catalog) -> anyhow::Result<Self> {
        Ok(Self {
            catalog
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
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
            let handle = BlockStreaming2::spawn(chain, &storage_nodes).await?;
        }

        Ok(())
    }
}
