use std::any::Any;
use std::sync::Arc;

use axum::routing::get;
use axum::routing::post;
use axum::Router;

use super::model;
use super::route::Context;
use super::API_ROOT_PATH;
use super::API_VERSION;
use crate::storeage::controller::postgres::PolkadotPostgresStorageImpl;

struct QueryHandle;

impl QueryHandle {
    pub async fn list_psql_tables(
        State(ctx): State<Context>,
    ) -> Result<Json<model::polkadot::ListPostgresTables>, (StatusCode, String)> {
        let controller = {
            let controllers = ctx.controllers.read().await;
            controllers.get("polkadot").unwrap().clone()
        };

        let query_tables =
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";
        let child = controller.get_child("postgres").await;
        let pg_impl = child.downcast::<PolkadotPostgresStorageImpl>().unwrap();
        let rows = pg_impl
            .base
            .pg_client
            .query(query_tables, &[])
            .await
            .unwrap();

        let mut response = model::polkadot::ListPostgresTables::default();
        for row in rows.into_iter() {
            let table_name: String = row.get(0);

            // let block_hash = format!("0x{}", hex::encode(&block_hash));
            // let parent_hash = format!("0x{}", hex::encode(&parent_hash));
            // let state_root = format!("0x{}", hex::encode(&state_root));
            // let extrinsics_root = format!("0x{}", hex::encode(&extrinsics_root));
            println!("table_name = {}", table_name);
            response.tables.push(table_name.clone());
            let query_table_info = "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = $1";
            let column_rows = pg_impl
                .base
                .pg_client
                .query(query_table_info, &[&table_name])
                .await
                .unwrap();

            let mut tables_info = vec![];
            for column_row in column_rows.into_iter() {
                let column_name: String = column_row.get(0);
                let data_type: String = column_row.get(1);
                tables_info.push(model::polkadot::TableInfo {
                    column_name,
                    data_type,
                });
            }

            response.tables_info.insert(table_name.clone(), tables_info);
        }

        Ok(Json(response))
    }

    pub async fn psql_block(
        State(ctx): State<Context>,
        Json(payload): Json<PsqlQueryPayload>,
    ) -> Result<Json<PolkadotPsqlBlockQueryResponse>, (StatusCode, String)> {
        assert_eq!(payload.chain.as_str(), "polkadot");
        assert_eq!(payload.table.as_str(), "block");

        let sql = payload.sql;
        let controller = {
            let controllers = ctx.controllers.read().await;
            controllers.get("polkadot").unwrap().clone()
        };

        let child = controller.get_child("postgres").await;
        let pg_impl = child.downcast::<PolkadotPostgresStorageImpl>().unwrap();
        let rows = pg_impl.base.pg_client.query(&sql, &[]).await.unwrap();

        let mut data = vec![];
        for row in rows.into_iter() {
            let block_number: i64 = row.get(0);
            let block_hash: String = row.get(1);
            let parent_hash: String = row.get(2);
            let state_root: String = row.get(3);
            let extrinsics_root: String = row.get(4);

            // let block_hash = format!("0x{}", hex::encode(&block_hash));
            // let parent_hash = format!("0x{}", hex::encode(&parent_hash));
            // let state_root = format!("0x{}", hex::encode(&state_root));
            // let extrinsics_root = format!("0x{}", hex::encode(&extrinsics_root));

            data.push(PolkadotBlock {
                block_number,
                block_hash,
                parent_hash,
                state_root,
                extrinsics_root,
            });
        }

        Ok(Json(PolkadotPsqlBlockQueryResponse { data }))
    }
}

pub struct PolkadotRouteBuild {
    path: String,
}

impl PolkadotRouteBuild {
    pub fn new() -> Self {
        Self {
            path: "polkadot".to_string(),
        }
    }

    pub fn build(self, mut router: Router<Context>) -> anyhow::Result<Router<Context>> {
        let base = self.base_path();
        router = router.route(
            format!("{}/query/psql/block", base).as_str(),
            post(QueryHandle::psql_block),
        );

        let list_psql_tables = format!("{}/psql/tables", base);
        tracing::info!("api register {}", list_psql_tables);
        router = router.route(&list_psql_tables, get(QueryHandle::list_psql_tables));

        Ok(router)
    }

    fn base_path(&self) -> String {
        // /apis/v1/polkadot
        format!("{}/{}/{}", API_ROOT_PATH, API_VERSION, self.path)
    }
}

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize)]
pub struct PsqlQueryPayload {
    /// Which chain to query.
    chain: String,
    /// Which table of chain to query.
    table: String,
    /// The sql statement to query.
    sql: String,
}
#[derive(Serialize)]
pub struct PolkadotBlock {
    pub block_number: i64,
    pub block_hash: String,
    pub parent_hash: String,
    pub state_root: String,
    pub extrinsics_root: String,
}

#[derive(Serialize)]
pub struct PolkadotPsqlBlockQueryResponse {
    data: Vec<PolkadotBlock>,
}

fn downcast_pg_impl(value: Arc<dyn Any + Send + Sync>) -> Arc<PolkadotPostgresStorageImpl> {
    value.downcast().unwrap()
}
