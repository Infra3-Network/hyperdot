use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::url::parse_storage_ops;
// use crate::types::BlockDescribe;
use crate::types::rpc::WriteBlockRequest;
use super::postgres::PolkadotPostgresStorageImpl;

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



    pub async fn write_polkadot_block(&self, req: &WriteBlockRequest<crate::types::polkadot::Block>) -> anyhow::Result<()> {
        match self.chain.as_str() {
            "polkadot" => {
                let rl = self.ops.read().await;
                for (name, any_ptr) in rl.iter() {
                    match name.as_str() {
                        "postgres" => {
                            let pg_impl = any_ptr.downcast_ref::<PolkadotPostgresStorageImpl>().unwrap();
                            pg_impl.write_block(req.clone()).await?;
                        },
                        _ => unimplemented!(),
                    }

                }
            },
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
