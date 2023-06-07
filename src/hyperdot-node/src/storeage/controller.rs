use std::sync::Arc;

use tokio::sync::RwLock;

use super::StorageOps;
use super::storage_url::parse_storage_ops;
use crate::types::BlockDescribe;

pub struct StorageControllerParams {
	pub store_urls: Vec<String>,
}

pub struct StorageController {
	params: StorageControllerParams,
	stores: Arc<RwLock<Vec<Box<dyn StorageOps>>>>,
}

impl StorageController {
	pub async fn new(params: StorageControllerParams) -> anyhow::Result<Self> {
		let stores = parse_storage_ops(&params.store_urls).await?;
		Ok(Self {
			params,
			stores: Arc::new(RwLock::new(stores)),
		})
	}

	pub async fn write_block(&self, blocks: &[BlockDescribe]) -> anyhow::Result<()> {
		let rl = self.stores.read().await;
		for store in rl.iter() {
			let _ = store.write_block(blocks).await?;
		}

		Ok(())
	}
}



