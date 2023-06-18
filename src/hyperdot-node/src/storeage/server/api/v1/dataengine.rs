use std::collections::HashMap;
use std::pin::Pin;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use futures::Future;

// use super::model::dataengine;
// use super::model::support;
// use super::model::system;
use super::route::Context;
use super::API_ROOT_PATH;
use super::API_VERSION;
use crate::storeage::engine::postgres::PolkadotPostgresStorageImpl;
use crate::storeage::engine::postgres::PostgresRows;

struct PostgresSchemeHandle;

impl PostgresSchemeHandle {
    pub async fn get_polkadot(
        State(ctx): State<Context>,
    ) -> Result<Json<dataengine::GetPostgresSchemeResponse>, StatusCode> {
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

        let mut tables = HashMap::new();
        for row in rows.into_iter() {
            let table_name: String = row.get(0);

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
                tables_info.push(dataengine::PostgresTableInfo {
                    column_name,
                    data_type,
                });
            }

            tables.insert(table_name, tables_info);
        }

        Ok(Json(dataengine::GetPostgresSchemeResponse {
            meta: support::ResponseMetadata::success("get polkadot postgres scheme success"),
            chain: "polkadot".to_string(),
            tables,
        }))
    }
}

pub struct DataEngineRouteBuilder {
    path: String,
}

impl DataEngineRouteBuilder {
    pub fn new() -> Self {
        Self {
            path: "dataengine".to_string(),
        }
    }

    pub fn build(self, router: Router<Context>) -> anyhow::Result<Router<Context>> {
        let base = self.base_path();

        let api_get_pg_polkadot_schemes = format!("{}/scheme/postgres/polkadot", base);
        tracing::info!("register api: {}", api_get_pg_polkadot_schemes);
        Ok(router.route(
            &api_get_pg_polkadot_schemes,
            get(PostgresSchemeHandle::get_polkadot),
        ))
    }

    fn base_path(&self) -> String {
        // /apis/v1/dataengine
        format!("{}/{}/{}", API_ROOT_PATH, API_VERSION, self.path)
    }
}
