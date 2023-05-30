use std::convert::TryInto;
use std::sync::Arc;

use anyhow::Result as AnyResult;
use bit_vec::BitVec;
use subxt::blocks::Block;
use subxt::config::substrate::H256;
use subxt::Config;
use subxt::PolkadotConfig;
use tokio_postgres::Error;
use tokio_postgres::NoTls;

#[async_trait::async_trait]

pub trait Storage<C: Config>: Send + Sync {
    async fn write_block_header(&self, header: C::Header);
}

#[derive(Clone)]
pub struct PolkadotStorage {
    client: Arc<tokio_postgres::Client>,
}

impl PolkadotStorage {
    pub async fn new() -> AnyResult<Self> {
        let (client, connection) = tokio_postgres::connect(
            "host=127.0.0.1 port=15721 user=noisepage_user password=noisepage_pass dbname=polkadot",
            NoTls,
        )
        .await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let client = Arc::new(client);
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl Storage<PolkadotConfig> for PolkadotStorage {
    async fn write_block_header(&self, header: <PolkadotConfig as Config>::Header) {
        // println!("write header {:?}", header);
        let block_number = header.number as i64;
        let parent_hash = BitVec::from_bytes(header.parent_hash.as_bytes());
        let block_hash = BitVec::from_bytes(header.parent_hash.as_bytes());
        let state_root = BitVec::from_bytes(header.state_root.as_bytes());
        let extrinsics_root = BitVec::from_bytes(header.extrinsics_root.as_bytes());

        self.client
        .execute(
            "INSERT INTO block_header (block_number, block_hash, parent_hash, state_root, extrinsics_root) VALUES ($1, $2, $3, $4, $5)",
            &[&block_number, &block_hash, &parent_hash, &state_root, &extrinsics_root],
        )
        .await.unwrap();

        let row = self
            .client
            .query_one(
                "SELECT parent_hash from block_header where block_number = $1",
                &[&block_number],
            )
            .await
            .unwrap();
        let value: BitVec = row.get(0);
        let data = &value.to_bytes().as_slice().try_into().unwrap();
        println!(
            "storage hash = {}, query hash = {}",
            header.parent_hash,
            H256::from(data)
        );
    }
}
