use std::sync::Arc;

use anyhow::Result as AnyResult;
use bit_vec::BitVec;

use tokio_postgres::Error;
use tokio_postgres::NoTls;
use tracing::debug;
use tracing::info;

use crate::types::BlockDescribe;

#[async_trait::async_trait]
pub trait BlockStorageOps
where Self: Send + Sync {
    async fn write_block(&self, blocks: &[BlockDescribe]) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait StorageOps: Send + Sync + BlockStorageOps {

}


// #[derive(Clone)]
// pub struct StorageOpsParams {
//     pub postgres_addr: String,
// }

// pub struct StorageOps {
//     // pg_client: Client,
// }

// impl StorageOps {
//     pub async fn new(params: StorageOpsParams) -> AnyResult<Self> {
//         // // "host=127.0.0.1 port=15721 user=noisepage_user password=noisepage_pass dbname=polkadot"
//         // let (client, connection) = tokio_postgres::connect(&params.postgres_addr, NoTls).await?;

//         // // The connection object performs the actual communication with the database,
//         // // so spawn it off to run on its own.
//         // tokio::spawn(async move {
//         //     if let Err(e) = connection.await {
//         //         panic!("connection error: {}", e);
//         //     }

//         //     info!("🌛 postgres connected");
//         // });
//         // let pg_client = client;
//         // Ok(Self { pg_client })
//         Ok(Self {})
//     }
// }

// impl StorageOps {
//     pub async fn write_block_header(&self, request: &BlockHeaderDescribe) -> AnyResult<()> {
//         let block_number = request.block_number as i64;
//         let parent_hash = BitVec::from_bytes(&request.block_hash);
//         let block_hash = BitVec::from_bytes(&request.parent_hash);
//         let state_root = BitVec::from_bytes(&request.state_root);
//         let extrinsics_root = BitVec::from_bytes(&request.extrinsics_root);

//         let _ = self.pg_client
//         .execute(
//             "INSERT INTO block_header (block_number, block_hash, parent_hash, state_root, extrinsics_root) VALUES ($1, $2, $3, $4, $5)",
//             &[&block_number, &block_hash, &parent_hash, &state_root, &extrinsics_root],
//         )
//         .await?;

//         info!("🚅 write block #{} to postgres", block_number);
//         Ok(())

//         // let row = self
//         //     .client
//         //     .query_one(
//         //         "SELECT parent_hash from block_header where block_number = $1",
//         //         &[&block_number],
//         //     )
//         //     .await
//         //     .unwrap();
//         // let value: BitVec = row.get(0);
//         // let data = &value.to_bytes().as_slice().try_into().unwrap();
//         // println!(
//         //     "storage hash = {}, query hash = {}",
//         //     header.parent_hash,
//         //     H256::from(data)
//         // );
//     }
// }
