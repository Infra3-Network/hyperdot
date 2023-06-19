use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use hyperdot_common_config::PublicChain;
use hyperdot_common_config::StorageConfig;
use hyperdot_common_config::StorageNodeConfig;
use tokio::sync::RwLock;

use super::engine::DataEngine;
use super::pg;
use super::postgres::PolkadotPostgresStorageImpl;
use super::url::parse_storage_ops;
use super::PgEngine;
use crate::types::rpc::WriteBlock;
// use crate::types::BlockDescribe;
use crate::types::rpc::WriteBlockRequest;

pub struct StorageControllerParams {
    pub chain: String,
    pub storages: Vec<String>,
}

pub struct StorageController {
    chain: String,
    ops: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>, // ops name -> ops pointer
}

impl StorageController {
    pub async fn new(params: StorageControllerParams) -> anyhow::Result<Self> {
        let ops = parse_storage_ops(&params.chain, &params.storages).await?;
        Ok(Self {
            chain: params.chain,
            ops: RwLock::new(ops),
        })
    }

    // pub async fn get_any_child(&self, name: &str) -> Arc<dyn Any> {
    //     let rl = self.ops.read().await;
    //     let child = rl.get(name).unwrap().clone();
    //     child
    // }

    pub async fn get_child(&self, name: &str) -> Arc<dyn Any + Send + Sync> {
        let rl = self.ops.read().await;
        let child = rl.get(name).unwrap();
        child.clone()
    }

    pub async fn write_polkadot_block(
        &self,
        req: &WriteBlockRequest<crate::types::polkadot::Block>,
    ) -> anyhow::Result<()> {
        match self.chain.as_str() {
            "polkadot" => {
                let rl = self.ops.read().await;
                for (name, any_ptr) in rl.iter() {
                    match name.as_str() {
                        "postgres" => {
                            let pg_impl = any_ptr
                                .downcast_ref::<PolkadotPostgresStorageImpl>()
                                .unwrap();
                            pg_impl.write_block(req.clone()).await?;
                        }
                        _ => unimplemented!(),
                    }
                }
            }
            _ => unimplemented!(),
        }

        Ok(())
        // for (_, ops_vptr) in rl.iter() {
        //     let transformed_blocks = ops_vptr.transform_block(blocks).await?;
        //     let _ = ops_vptr.write_block(transformed_blocks).await?;
        // }

        // Ok(())
    }
}

/// Data engione controller.
pub struct Controller {
    pub pg_engine: Option<Arc<PgEngine>>,
    engines: RwLock<Vec<Arc<dyn DataEngine>>>,
}

impl Controller {
    pub async fn async_new(
        engines: Vec<hyperdot_common_config::DataEngine>,
    ) -> anyhow::Result<Self> {
        let mut engs = vec![];

        let mut pg_engine = None;
        for engine in engines.iter() {
            if let Some(postgres_cfg) = engine.postgres.as_ref() {
                let engine = Arc::new(pg::PgEngine::new(postgres_cfg.clone()).await?);
                pg_engine = Some(engine.clone());
                let engine: Arc<dyn DataEngine> = engine;
                engs.push(engine)
            }
        }

        Ok(Self {
            pg_engine,
            engines: RwLock::new(engs),
        })
    }

    pub async fn get_pg_engine_or_error(&self) -> anyhow::Result<Arc<PgEngine>> {
        let pg_engine = match self.pg_engine.as_ref() {
            None => {
                // response.meta.set_code(ResponseCode::Error);
                // response
                //     .meta
                //     .set_reason(format!("postgres engine not found"));
                return Err(anyhow::anyhow!("Postgres data engine not found"));
            }
            Some(pg_engine) => Ok(pg_engine.clone()),
        };
    }

    pub async fn write_block(&self, mut req: WriteBlock) -> anyhow::Result<()> {
        // TODO: filter block at here.
        let engines = {
            let rl = self.engines.read().await;
            rl.clone()
        };

        /// One vec of blocks per datae ngine
        let engines_num = engines.len();
        let mut vblocks: Vec<Vec<Box<dyn Any + Send + Sync>>> = match req.chain.kind {
            PublicChain::Ethereum => unimplemented!(),
            PublicChain::Polkadot => {
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
