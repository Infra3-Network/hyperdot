use anyhow::anyhow;
use rust_decimal::prelude::Decimal;
use rust_decimal::prelude::FromPrimitive;
use subxt::ext::codec::Decode;
use tokio::task::JoinHandle;
use tokio_postgres::Client;
use tokio_postgres::NoTls;

use super::utils::FiveTopics;
use crate::runtime_api::polkadot;
use crate::runtime_api::GetName;
// use crate::types::BlockDescribe;
// use crate::types::BlockHeaderDescribe;
// use crate::types::EventDescribe;
use crate::types::rpc::WriteBlockRequest;

// use crate::types::ExtrinsicEventDescribe;

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
        format!(
            "host={} port={} user={} password={} dbname = {}",
            self.host, self.port, self.user, self.password, self.dbname,
        )
    }
}

pub struct PostgresStorage {
    params: PostgresStorageParams,
    pub pg_client: Client,
    pg_conn_handle: JoinHandle<anyhow::Result<()>>,
}

impl PostgresStorage {
    pub async fn new(params: PostgresStorageParams) -> anyhow::Result<Self> {
        tracing::info!(
            "🐘 PostgresStorage: try connecting {}:{}, dbname = {}",
            params.host,
            params.port,
            params.dbname
        );
        let (pg_client, connection) = tokio_postgres::connect(&params.to_url(), NoTls).await?;

        // The connection object performs the actual communication with the database,
        // // so spawn it off to run on its own.
        let pg_conn_handle = tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::error!("🐛 PostgresStorage: postgres connection error: {}", err);
                return Err(anyhow!("{}", err));
            }
            return Ok(());
        });

        tracing::info!(
            "🐘 PostgresStorage: connected {}:{}, dbname = {}",
            params.host,
            params.port,
            params.dbname
        );
        Ok(Self {
            params,
            pg_client,
            pg_conn_handle,
        })
    }
}

pub struct PolkadotPostgresStorageImpl {
    pub base: PostgresStorage,
}

impl PolkadotPostgresStorageImpl {
    async fn write_events(
        &self,
        header: &crate::types::polkadot::BlockHeader,
        events: &mut [crate::types::polkadot::EventDescribe],
    ) -> anyhow::Result<()> {
        let block_number = header.block_number as i64;
        let block_hash = &header.block_hash;
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
								pallet_index,
								event_name
							) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                            ON CONFLICT (block_number, index)
                            DO UPDATE SET
                                block_number = excluded.block_number,
                                block_hash = excluded.block_hash,
                                extrinsic_hash = excluded.extrinsic_hash,
                                data = excluded.data,
                                index = excluded.index,
                                topic0 = excluded.topic0,
                                topic1 = excluded.topic1,
                                topic2 = excluded.topic2,
                                topic3 = excluded.topic3,
                                topic4 = excluded.topic4,
                                phase = excluded.phase,
                                pallet_name = excluded.pallet_name,
                                event_name = excluded.event_name";

        let upsert_transfer_statemant = "INSERT INTO transfer (
                                block_number, 
                                block_hash, 
                                index, 
                                pallet_index, 
                                pallet_name, 
                                event_name, 
                                extrinsic_hash, 
                                \"from\", 
                                \"to\", 
                                amount, 
                                success
                            )VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)";
        // ON CONFLICT (block_number)
        // DO UPDATE SET
        //     block_hash = excluded.block_hash,
        //     index = excluded.index,
        //     pallet_index = excluded.pallet_index,
        //     pallet_name = excluded.pallet_name,
        //     hash = excluded.hash,
        //     \"from\" = excluded.from,
        //     \"to\" = excluded.to,
        //     amount = excluded.amount";

        let upsert_withdraw_statemant = "INSERT INTO withdraw (
                                block_number, 
                                block_hash, 
                                index, 
                                pallet_index, 
                                pallet_name, 
                                event_name,
                                extrinsic_hash, 
                                who, 
                                amount, 
                                success
                            )VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   ON CONFLICT (block_number, index)
                   DO UPDATE SET
                        block_hash = excluded.block_hash,
                        index = excluded.index,
                        pallet_index = excluded.pallet_index,
                        pallet_name = excluded.pallet_name,
                        hash = excluded.hash,
                        who = excluded.who,
                        amount = excluded.amount";

        // ON CONFLICT (block_number)
        // DO UPDATE SET
        //     block_hash = excluded.block_hash,
        //     index = excluded.index,
        //     pallet_index = excluded.pallet_index,
        //     pallet_name = excluded.pallet_name,
        //     hash = excluded.hash,
        //     who = excluded.who,
        //     amount = excluded.amount";

        let mut stmts = vec![];
        for _ in 0..events.len() {
            // FIXME: make stream concurrent
            let stmt = self
                .base
                .pg_client
                .prepare(upsert_raw_event_statemant)
                .await?;
            stmts.push(stmt);
        }

        let mut extrinsic_success = false;
        for (i, event_desc) in events.iter_mut().enumerate() {
            let root_event = polkadot::Event::decode(&mut event_desc.root_bytes.as_ref()).unwrap();
            match root_event {
                polkadot::Event::System(system_event) => match system_event {
                    polkadot::system::Event::ExtrinsicSuccess { .. } => extrinsic_success = true,
                    _ => {}
                },
                _ => {}
            }
        }

        for (i, event_desc) in events.iter_mut().enumerate() {
            let root_event = polkadot::Event::decode(&mut event_desc.root_bytes.as_ref())?;

            let (_, event_name) = root_event.name();
            let five_topics = FiveTopics::from(&event_desc.topics);
            let block_time: i64 = 0;
            // FIXME: make stream concurrent
            let rows = self
                .base
                .pg_client
                .execute(&stmts[i], &[
                    &block_number,
                    &block_hash,
                    &block_time, // FIXME: block_time
                    &event_desc.extrinsic_hash,
                    &event_desc.data,
                    &(event_desc.index as i32),
                    &five_topics.t0,
                    &five_topics.t1,
                    &five_topics.t2,
                    &five_topics.t3,
                    &five_topics.t4,
                    &event_desc.phase.to_string(),
                    &event_desc.pallet_name,
                    &(event_desc.pallet_index as i16),
                    &event_name,
                ])
                .await?;
            // Check if the row count is 1, indicating a successful insert
            if rows == 1 {
                println!("raw_event inserted successfully");
            } else {
                println!("raw_event updated successfully");
            }

            match root_event {
                polkadot::Event::Balances(balance_event) => match balance_event {
                    polkadot::balances::Event::Transfer { from, to, amount } => {
                        let event_name = "Transfer";
                        let from = from.0;
                        let to = to.0;
                        let amount = Decimal::from_u128(amount)
                            .expect("parse transfer u128 to decimal error");

                        let rows = self
                            .base
                            .pg_client
                            .execute(upsert_transfer_statemant, &[
                                &block_number,
                                &block_hash,
                                &(event_desc.index as i32),
                                &(event_desc.pallet_index as i16),
                                &event_desc.pallet_name,
                                &event_name,
                                &event_desc.extrinsic_hash,
                                &from,
                                &to,
                                &amount,
                                &extrinsic_success,
                            ])
                            .await?; // FIXME: not fault process

                        if rows == 1 {
                            println!("insert transfer #{} success", block_number);
                        }
                    }
                    polkadot::balances::Event::Withdraw { who, amount } => {
                        let event_name = "Withdraw";
                        let who = who.0;
                        let amount = Decimal::from_u128(amount)
                            .expect("parse withdraw u128 to decimal error");

                        let rows = self
                            .base
                            .pg_client
                            .execute(upsert_withdraw_statemant, &[
                                &block_number,
                                &block_hash,
                                &(event_desc.index as i32),
                                &(event_desc.pallet_index as i16),
                                &event_desc.pallet_name,
                                &event_name,
                                &event_desc.extrinsic_hash,
                                &who,
                                &amount,
                                &extrinsic_success,
                            ])
                            .await?; // FIXME: not fault process

                        if rows == 1 {
                            println!("insert withdraw #{} success", block_number);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }
}

impl PolkadotPostgresStorageImpl {
    pub async fn write_block(
        &self,
        req: WriteBlockRequest<crate::types::polkadot::Block>,
    ) -> anyhow::Result<()> {
        let mut blocks = req
            .blocks
            .into_iter()
            .map(|b| b.clone())
            .collect::<Vec<_>>();
        tracing::info!(
            "🐘 PostgresStorage: writing blocks #{:?}",
            blocks
                .iter()
                .map(|blk| format!("#{}", blk.header.block_number))
                .collect::<Vec<_>>()
        );

        let upsert_statement =
            "INSERT INTO block (block_number, block_hash, parent_hash, state_root, extrinsics_root)
                            VALUES ($1, $2, $3, $4, $5)
                            ON CONFLICT (block_number)
                            DO UPDATE SET
                                block_hash = excluded.block_hash,
                                parent_hash = excluded.parent_hash,
                                state_root = excluded.state_root,
                                extrinsics_root = excluded.extrinsics_root";

        for block in blocks.iter_mut() {
            let block_number = block.header.block_number as i64;
            let block_hash_hex = format!("0x{}", hex::encode(&block.header.block_hash));
            let parent_hash_hex = format!("0x{}", hex::encode(&block.header.parent_hash));
            let state_root_hex = format!("0x{}", hex::encode(&block.header.state_root));
            let extrinsics_root_hex = format!("0x{}", hex::encode(&block.header.extrinsics_root));
            let rows = self
                .base
                .pg_client
                .execute(upsert_statement, &[
                    &block_number,
                    &block_hash_hex,
                    &parent_hash_hex,
                    &state_root_hex,
                    &extrinsics_root_hex,
                ])
                .await?;
            // Check if the row count is 1, indicating a successful insert
            if rows == 1 {
                println!("block inserted successfully");
            } else {
                println!("block updated successfully");
            }

            for extrinsic in block.extrinsics.iter_mut() {
                if extrinsic.events.is_empty() {
                    tracing::info!("{} events empty, skip it", extrinsic.pallet_name);
                    continue;
                }

                self.write_events(&block.header, &mut extrinsic.events)
                    .await?;
            }
        }
        Ok(())
    }
}
