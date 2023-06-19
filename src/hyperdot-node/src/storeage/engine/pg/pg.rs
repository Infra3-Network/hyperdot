use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use hyperdot_core::types::PostgresDataEngine;
use hyperdot_core::types::PostgresDataEngineConnection;
use hyperdot_core::types::PostgresDataEngineForChain;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_postgres::Client;
use tokio_postgres::NoTls;

use super::super::engine::DataEngine;
use super::writer::SubstrateWriter;
use crate::types::block::polkadot_chain;

pub struct ConnectionState {
    pub client: Client,
    pub used_connection: PostgresDataEngineConnection,
    pub support_chain: PostgresDataEngineForChain,
    pub connection_config: tokio_postgres::Config,
    pub connection_handle: JoinHandle<anyhow::Result<()>>,
}

pub struct PgEngine {
    // support chain name of connection map to state
    connections: RwLock<HashMap<String, Arc<ConnectionState>>>,
}

impl PgEngine {
    pub async fn new(engine: PostgresDataEngine) -> anyhow::Result<Self> {
        let mut connections = HashMap::new();
        for support_chain in engine.support_chains.iter() {
            if !support_chain.enabled {
                tracing::info!(
                    "💁 {}: skipped not enabled for postgres data engine",
                    support_chain.name
                );
                continue;
            }

            let used_connection = match engine
                .connections
                .iter()
                .find(|c| c.name == support_chain.use_connection)
            {
                None => {
                    tracing::error!(
                        "💁 {}: using connection name = {} for postgres, but not found in postgres.connections, please check config",
                        support_chain.name,
                        support_chain.use_connection,
                    );
                    continue;
                }
                Some(c) => c,
            };
            let used_connection_name = used_connection.name.clone();

            let mut connection_config = tokio_postgres::Config::default();
            connection_config.user(&used_connection.username);
            connection_config.password(&used_connection.password);
            connection_config.host(&used_connection.host);
            connection_config.port(used_connection.port);
            connection_config.dbname(&support_chain.dbname);
            let (client, connection) = match connection_config.connect(NoTls).await {
                Err(err) => {
                    tracing::error!(
                        "💔 {}: connection name = {} connect postgres error: {}",
                        support_chain.name,
                        used_connection.name,
                        err
                    );
                    continue;
                }
                Ok(res) => res,
            };

            // TODO: check database tables, if not exists maybe consider init it.
            let connection_handle = tokio::spawn(async move {
                // TODO: consider re-connection
                if let Err(err) = connection.await {
                    tracing::error!(
                        "🐛 {}: postgres connection has broken: {}",
                        used_connection_name,
                        err
                    );
                    return Err(anyhow!("{}", err));
                }
                return Ok(());
            });

            tracing::info!(
                "🙅 {}: postgres data engine connected at dbname({})",
                support_chain.name,
                support_chain.dbname,
            );

            connections.insert(
                support_chain.name.to_string(),
                Arc::new(ConnectionState {
                    support_chain: support_chain.clone(),
                    used_connection: used_connection.clone(),
                    connection_config,
                    client,
                    connection_handle,
                }),
            );
        }

        Ok(Self {
            connections: RwLock::new(connections),
        })
    }

    pub async fn get_conn_state_for_chain(
        &self,
        chain: &str,
    ) -> anyhow::Result<Arc<ConnectionState>> {
        let rl = self.connections.read().await;
        match rl.get(chain) {
            None => {
                return Err(anyhow::anyhow!("Postgres not support chain({})", chain));
            }
            Some(conn) => Ok(conn.clone()),
        }
    }
    pub async fn write_block_internal(
        &self,
        chain: &str,
        block: Box<dyn Any + Send + Sync>,
    ) -> anyhow::Result<()> {
        let conn_state = self.get_conn_state_for_chain(chain).await?;
        let block = block.downcast::<polkadot_chain::Block>().unwrap();
        SubstrateWriter::write_block(conn_state.clone(), *block).await
    }

    /// Run query sql for chain.
    pub async fn query(
        &self,
        chain: &str,
        sql: &str,
    ) -> anyhow::Result<hyperdot_core::types::PostgresRows> {
        let conn_state = self.get_conn_state_for_chain(chain).await?;
        let rows = conn_state.client.query(sql, &[]).await.map_err(|err| {
            anyhow::anyhow!(
                "Postgres data engine run sql({}) for chain({}) error:{}",
                sql,
                chain,
                err
            )
        })?;

        hyperdot_core::types::PostgresRows::try_from(rows)
    }

    // pub async fn write_block_for_polkadot_chain(
    //     &self,
    //     chain: &Chain,
    //     block: Box<dyn Any + Send + Sync>,
    // ) -> anyhow::Result<()> {
    //     let runtime_name = chain
    //         .polkadot_runtime
    //         .as_ref()
    //         .map_or("substrate".to_string(), |r| r.config.clone());
    //     match runtime_name.as_str() {
    //         "polkadot" => {
    //             let block = block.downcast::<polkadot_chain::Block>().unwrap();
    //             self.write_block_for_polkadot_runtime_polkadot(chain, *block)
    //                 .await
    //         }
    //         "substrate" => {
    //             let block = block.downcast::<polkadot_chain::Block>().unwrap();
    //             self.write_block_for_polkadot_runtime_substrate(chain, *block)
    //                 .await
    //         }

    //         _ => unimplemented!(),
    //     }
    // }

    // pub async fn write_block_for_polkadot_runtime_polkadot(
    //     &self,
    //     chain: &Chain,
    //     block: polkadot_chain::Block,
    // ) -> anyhow::Result<()> {
    //     let state = match self.connections.get(&chain.name) {
    //         None => {
    //             tracing::error!(
    //                 "👉 Postgres can't writ block({}) for chain({}), connection not found",
    //                 block.header.block_number,
    //                 chain.name
    //             );
    //             return Ok(());
    //         }
    //         Some(state) => state,
    //     };

    //     todo!()
    // }

    // pub async fn write_block_for_polkadot_runtime_substrate(
    //     &self,
    //     chain: &Chain,
    //     block: polkadot_chain::Block,
    // ) -> anyhow::Result<()> {
    //     todo!()
    // }
}

#[async_trait::async_trait]
impl DataEngine for PgEngine {
    fn name(&self) -> String {
        "Postgres".to_string()
    }

    async fn write_block(
        &self,
        chain: String,
        blocks: Vec<Box<dyn Any + Send + Sync>>,
    ) -> anyhow::Result<()> {
        for block in blocks {
            match self.write_block_internal(&chain, block).await {
                Err(err) => {
                    tracing::error!("write block error: {}", err);
                    continue;
                }
                Ok(_) => {}
            };
        }

        Ok(())
    }
}
