use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use hyperdot_core::protocols::ListDataEngineResponse;
use hyperdot_core::protocols::ListDataEngineResquest;
use hyperdot_core::protocols::ResponseCode;
use hyperdot_core::types::ChainInfo;
use hyperdot_core::types::EngineInfo;

// use super::model::support;
// use super::model::system;
// use super::model::system::Chain;
use super::route::Context;
// use super::service;
use super::API_ROOT_PATH;
use super::API_VERSION;

async fn list_dataengines(
    State(ctx): State<Context>,
) -> Result<Json<ListDataEngineResponse>, StatusCode> {
    // TODO: make cfg from rocksdb.
    let mut response = ListDataEngineResponse::default();
    if ctx.cfg.data_engines.is_empty() {
        response.header.set_code(ResponseCode::Error);
        response.header.set_reason(format!("data engines is empty"));
        return Ok(Json(response));
    }

    for de in ctx.cfg.data_engines.iter() {
        if let Some(pg) = de.postgres.as_ref() {
            let mut engine_info = EngineInfo::default();
            for sc in pg.support_chains.iter() {
                engine_info
                    .support_chains
                    .insert(sc.name, ChainInfo { name: sc.name });
            }
            response.engines.insert("Postgres".to_string(), engine_info); // TODO: make enum
        }
    }
    response
        .header
        .set_success_msg(format!("list data engines success"));
    return Ok(Json(response));
}

pub struct SystemRouteBuilder {
    path: String,
}

impl SystemRouteBuilder {
    pub fn new() -> Self {
        Self {
            path: "system".to_string(),
        }
    }

    pub fn build(self, router: Router<Context>) -> anyhow::Result<Router<Context>> {
        let base = self.base_path();

        let api_list_dataengines = format!("{}/dataengines", base);
        tracing::info!("register api: {}", api_list_dataengines);

        Ok(router.route(&api_list_dataengines, get(list_dataengines)))
    }

    fn base_path(&self) -> String {
        // /apis/v1/system
        format!("{}/{}/{}", API_ROOT_PATH, API_VERSION, self.path)
    }
}
