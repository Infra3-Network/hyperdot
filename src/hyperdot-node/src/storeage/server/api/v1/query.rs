use std::any::Any;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use hyperdot_core::protocols::QueryPostgresRequest;
use hyperdot_core::protocols::QueryPostgresResponse;
use hyperdot_core::protocols::ResponseCode;

// use super::model;
// use super::model::support::ResponseCode;
use super::route::Context;
use super::API_ROOT_PATH;
use super::API_VERSION;
// use crate::storeage::engine::postgres::PolkadotPostgresStorageImpl;
// use crate::storeage::engine::postgres::PostgresRows;

struct PostgresQueryHandle;

impl PostgresQueryHandle {
    pub async fn run(
        State(ctx): State<Context>,
        Json(request): Json<QueryPostgresRequest>,
    ) -> Result<Json<QueryPostgresResponse>, (StatusCode, String)> {
        let mut response = QueryPostgresResponse::default();
        // if !core::model::SUPPORT_DATA_ENGINES.is_support(&request.engine) {
        //     response.meta.set_code(ResponseCode::Error);
        //     response
        //         .meta
        //         .set_reason(format!("{} engine not support", request.engine));
        //     return Ok(Json(response));
        // }

        // if !core::model::SUPPORT_DATA_ENGINES.is_support_chain(&request.engine, &request.chain) {
        //     response.meta.set_code(ResponseCode::Error);
        //     response.meta.set_reason(format!(
        //         "{} chain not support at engine {}",
        //         request.chain, request.engine
        //     ));
        //     return Ok(Json(response));
        // }

        if request.query.is_empty() {
            response.meta.set_code(ResponseCode::Error);
            response.meta.set_reason(format!("query is empty"));
            return Ok(Json(response));
        }

        let pg_engine = match ctx.engine_controller.get_pg_engine().await {
            Err(err) => {
                response.meta.set_error(err.to_string());
                return Ok(Json(response));
            }
            Ok(pg_engine) => pg_engine,
        };

        match pg_engine.query(&request.chain, &request.query).await {
            Err(err) => {
                response.meta.set_error(err.to_string());
                return Ok(Json(response));
            }

            Ok(res) => {
                response.rows = res;
                return Ok(Json(response));
            }
        }
        // match request.chain.as_str() {
        //     "polkadot" => {
        //         let controller = {
        //             let controllers = ctx.controllers.read().await;
        //             controllers.get("polkadot").unwrap().clone()
        //         };

        //         let child = controller.get_child("postgres").await;
        //         let pg_impl = child.downcast::<PolkadotPostgresStorageImpl>().unwrap();
        //         let rows = pg_impl
        //             .base
        //             .pg_client
        //             .query(&request.query, &[])
        //             .await
        //             .unwrap();

        //         response.rows = PostgresRows::try_from(rows).unwrap();
        //         return Ok(Json(response));
        //     }
        //     _ => {
        //         response.meta.set_code(ResponseCode::Error);
        //         response
        //             .meta
        //             .set_reason(format!("{} chain not support", request.chain));
        //         return Ok(Json(response));
        //     }
        // }
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
