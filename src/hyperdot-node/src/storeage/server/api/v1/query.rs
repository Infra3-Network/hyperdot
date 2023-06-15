use std::any::Any;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;

use super::core;
use super::core::ResponseCode;
use super::model;
use super::route::Context;
use super::API_ROOT_PATH;
use super::API_VERSION;
use crate::storeage::controller::postgres::PolkadotPostgresStorageImpl;
use crate::storeage::controller::postgres::PostgresRows;

struct PostgresQueryHandle;

impl PostgresQueryHandle {
    pub async fn run(
        State(ctx): State<Context>,
        Json(payload): Json<model::query::RunPostgresQueryPayload>,
    ) -> Result<Json<model::query::RunPostgresQueryResponse>, (StatusCode, String)> {
        let mut response = model::query::RunPostgresQueryResponse::default();
        if !core::model::SUPPORT_DATA_ENGINES.is_support(&payload.engine) {
            response.meta.set_code(ResponseCode::Error);
            response
                .meta
                .set_reason(format!("{} engine not support", payload.engine));
            return Ok(Json(response));
        }

        if !core::model::SUPPORT_DATA_ENGINES.is_support_chain(&payload.engine, &payload.chain) {
            response.meta.set_code(ResponseCode::Error);
            response.meta.set_reason(format!(
                "{} chain not support at engine {}",
                payload.chain, payload.engine
            ));
            return Ok(Json(response));
        }

        if payload.query.is_empty() {
            response.meta.set_code(ResponseCode::Error);
            response.meta.set_reason(format!("query is empty"));
            return Ok(Json(response));
        }

        match payload.chain.as_str() {
            "polkadot" => {
                let controller = {
                    let controllers = ctx.controllers.read().await;
                    controllers.get("polkadot").unwrap().clone()
                };

                let child = controller.get_child("postgres").await;
                let pg_impl = child.downcast::<PolkadotPostgresStorageImpl>().unwrap();
                let rows = pg_impl
                    .base
                    .pg_client
                    .query(&payload.query, &[])
                    .await
                    .unwrap();

                response.rows = PostgresRows::try_from(rows).unwrap();
                return Ok(Json(response));
            }
            _ => {
                response.meta.set_code(ResponseCode::Error);
                response
                    .meta
                    .set_reason(format!("{} chain not support", payload.chain));
                return Ok(Json(response));
            }
        }
    }
}

pub struct QueryRouteBuilder {
    path: String,
}

impl QueryRouteBuilder {
    pub fn new() -> Self {
        Self {
            path: "query".to_string(),
        }
    }

    pub fn build(self, mut router: Router<Context>) -> anyhow::Result<Router<Context>> {
        let base = self.base_path();

        let run_posrgres = format!("{}/run/postgres", base);
        tracing::info!("register api: {}", run_posrgres);

        router = router.route(run_posrgres.as_str(), post(PostgresQueryHandle::run));
        Ok(router)
    }

    fn base_path(&self) -> String {
        // /apis/v1/query
        format!("{}/{}/{}", API_ROOT_PATH, API_VERSION, self.path)
    }
}
