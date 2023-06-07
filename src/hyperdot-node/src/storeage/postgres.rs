use tokio_postgres::Client;
use tokio_postgres::Error;
use tokio_postgres::NoTls;
use tokio::task::JoinHandle;
use anyhow::anyhow;

use super::StorageOps;
use super::BlockStorageOps;

use crate::types::BlockDescribe;

#[derive(Debug)]
pub struct PostgresStorageParams {
	// TODO: postgres://{raw} in uper
	pub host: String,
	pub port: u16,
	pub user: String,
	pub password: String,
	pub dbname: String,
}

impl PostgresStorageParams {
	pub fn to_url(&self) -> String {
		format!("host={} port={} user={} password={} dbname = {}", 
			self.host,
			self.port,
			self.user,
			self.password,
			self.dbname,
		)
	}
}


pub struct PostgresStorage {
	params: PostgresStorageParams,
	pg_client: Client,
	pg_conn_handle: JoinHandle<anyhow::Result<()>>,
}

impl PostgresStorage {
	pub async fn  new(params: PostgresStorageParams) -> anyhow::Result<Self> {
		tracing::info!("🐘 PostgresStorage: try connecting {}:{}, dbname = {}", params.host, params.port, params.dbname);
        let (pg_client, connection) = tokio_postgres::connect(&params.to_url(), NoTls).await?;

        // The connection object performs the actual communication with the database,
        // // so spawn it off to run on its own.
        let pg_conn_handle = tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::error!("🐛 PostgresStorage: postgres connection error: {}", err);
                return Err(anyhow!("{}", err))
            }
            return Ok(())
        });

        tracing::info!("🐘 PostgresStorage: connected {}:{}, dbname = {}", params.host, params.port, params.dbname);
        Ok(Self { 
        	params,
        	pg_client,
        	pg_conn_handle,
         })
	}
}

#[async_trait::async_trait]
impl BlockStorageOps for PostgresStorage {
	async fn write_block(&self, blocks: &[BlockDescribe]) -> anyhow::Result<()> {
		tracing::info!("🐘 PostgresStorage: writing blocks #{:?}", blocks.iter().map(|blk| format!("#{}", blk.header.block_number)).collect::<Vec<_>>());
	
		// TODO: using pipeline
		let upsert_statement = "INSERT INTO block (block_number, block_hash, parent_hash, state_root, extrinsics_root)
                            VALUES ($1, $2, $3, $4, $5)
                            ON CONFLICT (block_number)
                            DO UPDATE SET
                                block_hash = excluded.block_hash,
                                parent_hash = excluded.parent_hash,
                                state_root = excluded.state_root,
                                extrinsics_root = excluded.extrinsics_root";
		
		for block in blocks.iter() {
			let rows = self.pg_client.execute(upsert_statement, &[
				&(block.header.block_number as i64), 
				&block.header.block_hash, 
				&block.header.parent_hash, 
				&block.header.state_root, 
				&block.header.extrinsics_root],
			).await?;
			// Check if the row count is 1, indicating a successful insert
		    if rows == 1 {
		        println!("Data inserted successfully");
		    } else {
		        println!("Data updated successfully");
		    }
		}
		Ok(())
	}
}

#[async_trait::async_trait]
impl StorageOps for PostgresStorage {

}
