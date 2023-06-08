use std::any::Any;

use tokio_postgres::Client;
use tokio_postgres::Error;
use tokio_postgres::NoTls;
use tokio::task::JoinHandle;
use anyhow::anyhow;
use rust_decimal::prelude::Decimal;
use rust_decimal::prelude::FromPrimitive;

use super::StorageOps;
use super::BlockStorageOps;

use crate::types::BlockDescribe;
use crate::types::ExtrinsicEventDescribe;

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
	async fn transform_block(&self, blocks: &[BlockDescribe]) -> anyhow::Result<Vec<Box<dyn Any + Send + Sync>>> {
		let mut results = vec![];
		for block in blocks.iter() {
			let anyed: Box<dyn Any + Send + Sync> = Box::new(block.clone());
			results.push(anyed);
		}

		Ok(results)
	}

	async fn write_block(&self, blocks: Vec<Box<dyn Any + Send + Sync>>) -> anyhow::Result<()> {
		let blocks = blocks.into_iter().map(|b| b.downcast::<BlockDescribe>().unwrap() ).collect::<Vec<_>>();
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
		
		let upsert_transfer_statemant = "INSERT INTO transfer (block_number, block_hash, index, pallet_index, pallet_name, hash, \"from\", \"to\", amount, success)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                            ON CONFLICT (block_number)
                            DO UPDATE SET
                                block_hash = excluded.block_hash,
                                index = excluded.index,
                                pallet_index = excluded.pallet_index,
                                pallet_name = excluded.pallet_name,
                                hash = excluded.hash,
                                \"from\" = excluded.from,
                                \"to\" = excluded.to,
                                amount = excluded.amount";

		let upsert_withdraw_statemant = "INSERT INTO withdraw (block_number, block_hash, index, pallet_index, pallet_name, hash, who, amount, success)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                            ON CONFLICT (block_number)
                            DO UPDATE SET
                                block_hash = excluded.block_hash,
                                index = excluded.index,
                                pallet_index = excluded.pallet_index,
                                pallet_name = excluded.pallet_name,
                                hash = excluded.hash,
                                who = excluded.who,
                                amount = excluded.amount";

		let upsert_raw_event_statemant = "INSERT INTO raw_event (
								block_number, 
								block_hash, 
								block_time,
								extrinsic_hash,
								data,
								index,
								topic0, 
								topic1,
								topic2, 
								topic3,
								topic4,
								phase,
								pallet_name,
								pallet_index
							) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                            ON CONFLICT (block_number)
                            DO UPDATE SET
                                block_number = excluded.block_number,
                                block_hash = excluded.block_hash, 
                                extrinsic_hash = excluded.extrinsic_hash";


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
		        println!("block inserted successfully");
		    } else {
		        println!("block updated successfully");
		    }

		    for raw_event in block.raw_events.iter() {
		    	let rows = self.pg_client.execute(upsert_raw_event_statemant, &[
					&(raw_event.block_number as i64), 
					&raw_event.block_hash,
					&(raw_event.block_time as i64), 
					&raw_event.extrinsic_hash,
					&raw_event.data,
					&(raw_event.index as i32), 
					&raw_event.topic0,
					&raw_event.topic1,
					&raw_event.topic2,
					&raw_event.topic3,
					&raw_event.topic4,			
					&raw_event.phase.to_string(),
					&raw_event.pallet_name,
					&(raw_event.pallet_index as i16),
				]).await?;
				// Check if the row count is 1, indicating a successful insert
			    if rows == 1 {
			        println!("raw_event inserted successfully");
			    } else {
			        println!("raw_event updated successfully");
			    }

		    }

		    for extrinsic in block.extrinsics.iter() {
		    	if extrinsic.events.is_empty() {
		    		tracing::info!("{} events empty, skip it", extrinsic.pallet_name);
		    		continue
		    	}

		    	for extrinsic_event in extrinsic.events.iter() {
		    		match extrinsic_event {
		    			ExtrinsicEventDescribe::Transfer(transfer) => {
		    				let from: [u8; 32]  = transfer.from.clone();
		    				let to: [u8; 32]  = transfer.to.clone();
		    				let amount = Decimal::from_u128(transfer.amount).expect("parse transfer u128 to decimal error");
		    				let rows = self.pg_client.execute(upsert_transfer_statemant, &[
		    					&(block.header.block_number as i64), 
								&block.header.block_hash, 
								&(extrinsic.index as i32),
								&(extrinsic.pallet_index as i16),
								&extrinsic.pallet_name,
								&extrinsic.hash,
								&from,
								&to,
								&amount,
								&transfer.success,
		    				]).await?;
		    				// Check if the row count is 1, indicating a successful insert
						    if rows == 1 {
						        println!("transfer inserted successfully");
						    } else {
						        println!("transfer updated successfully");
						    }
		    			},
		    			ExtrinsicEventDescribe::Withdraw(withdraw) => {
		    				let amount = Decimal::from_u128(withdraw.amount).expect("parse withdraw u128 to decimal error");
		    				let who: [u8;32] = withdraw.who.clone();
		    				let rows = self.pg_client.execute(upsert_withdraw_statemant, &[
		    					&(block.header.block_number as i64), 
								&block.header.block_hash, 
								&(extrinsic.index as i32),
								&(extrinsic.pallet_index as i16),
								&extrinsic.pallet_name,
								&extrinsic.hash,
								&who,
								&amount,
								&withdraw.success,
		    				]).await?;
		    				// Check if the row count is 1, indicating a successful insert
						    if rows == 1 {
						        println!("withdraw inserted successfully");
						    } else {
						        println!("withdraw updated successfully");
						    }
		    			},
		    			_ => {},
		    		}
		    	}
		    }
		}
		Ok(())
	}
}

#[async_trait::async_trait]
impl StorageOps for PostgresStorage {

}
