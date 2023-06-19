use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use hyperdot_core::config::StorageConfig;
use hyperdot_core::config::StorageNodeConfig;
use hyperdot_core::types::ChainKind;
use hyperdot_core::types::DataEngineInfo;
use hyperdot_core::types::DataEngineKind;
// use hyperdot_common_config::PublicChain;
// use hyperdot_common_config::StorageConfig;
// use hyperdot_common_config::StorageNodeConfig;
use tokio::sync::RwLock;

use super::engine::DataEngine;
use super::pg;
// use super::url::parse_storage_ops;
use super::PgEngine;
use crate::types::rpc::WriteBlock;
// use crate::types::BlockDescribe;

/// Data engione controller.
pub struct Controller {
    pg_engine: Option<Arc<PgEngine>>,
    engines: RwLock<Vec<Arc<dyn DataEngine>>>,
}

impl Controller {
    pub async fn async_new(engines_info: Vec<DataEngineInfo>) -> anyhow::Result<Self> {
        let mut pg_engine = None;
        let mut dyn_engines = vec![];
        for engine_info in engines_info.iter() {
            match engine_info.kind {
                DataEngineKind::Postgres => match engine_info.postgres.as_ref() {
                    None => return Err(anyhow!("postgres data-engine config is none")),
                    Some(postgres_cfg) => {
                        let engine = Arc::new(pg::PgEngine::new(postgres_cfg.clone()).await?);
                        pg_engine = Some(engine.clone());
                        let engine: Arc<dyn DataEngine> = engine;
                        dyn_engines.push(engine);
                    }
                },
            }
        }

        Ok(Self {
            pg_engine,
            engines: RwLock::new(dyn_engines),
        })
    }

    pub async fn get_pg_engine(&self) -> anyhow::Result<Arc<PgEngine>> {
        match self.pg_engine.as_ref() {
            None => Err(anyhow::anyhow!(
                "postgres data engine not found in controller"
            )),

            Some(pg_engine) => Ok(pg_engine.clone()),
        }
    }

    pub async fn write_block(&self, mut req: WriteBlock) -> anyhow::Result<()> {
        // TODO: filter block at here.
        let engines = {
            let rl = self.engines.read().await;
            rl.clone()
        };

        /// One vec of blocks per datae ngine
        let engines_num = engines.len();
        let mut vblocks: Vec<Vec<Box<dyn Any + Send + Sync>>> = match req.chain_kind {
            ChainKind::Ethereum => unimplemented!(),
            ChainKind::Polkadot => {
                if req.polkadot_blocks.is_none() {
                    return Err(anyhow::anyhow!("polkadot chain polkadot block not found"));
                }

                let mut vblocks = vec![];
                let blocks = req.polkadot_blocks.take().unwrap();
                for _ in 0..engines_num {
                    let mut bs: Vec<Box<dyn Any + Send + Sync>> = vec![];
                    for block in blocks.iter() {
                        bs.push(Box::new(block.clone()))
                    }
                    vblocks.push(bs)
                }
                vblocks
            }
        };

        for (i, engine) in engines.iter().enumerate() {
            let blocks = vblocks.swap_remove(i);
            match engine.write_block(req.chain.clone(), blocks).await {
                Err(err) => {
                    tracing::error!("🍼 engine({}) write_block error: {}", engine.name(), err);
                    continue;
                }
                Ok(_) => {
                    tracing::info!("engine({}) write block", engine.name())
                }
            }
        }

        Ok(())
    }
}
