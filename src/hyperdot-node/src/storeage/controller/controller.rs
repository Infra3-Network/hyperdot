use std::sync::Arc;

use tokio::sync::RwLock;

use super::ops::StorageOps;
use super::url::parse_storage_ops;
use crate::types::BlockDescribe;

pub struct StorageControllerParams {
    pub chain: String,
    pub storages: Vec<String>,
}

pub struct StorageController {
    stores: Arc<RwLock<Vec<Box<dyn StorageOps>>>>,
}

impl StorageController {
    pub async fn new(params: StorageControllerParams) -> anyhow::Result<Self> {
        let stores = parse_storage_ops(&params.chain, &params.storages).await?;
        Ok(Self {
            stores: Arc::new(RwLock::new(stores)),
        })
    }

    pub async fn write_block(&self, blocks: &[BlockDescribe]) -> anyhow::Result<()> {
        let rl = self.stores.read().await;
        for store in rl.iter() {
            let transformed_blocks = store.transform_block(blocks).await?;
            let _ = store.write_block(transformed_blocks).await?;
        }

        Ok(())
    }
}
