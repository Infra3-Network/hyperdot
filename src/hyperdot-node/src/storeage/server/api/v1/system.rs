use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;

use super::model::support;
use super::model::system;
use super::route::Context;
use super::API_ROOT_PATH;
use super::API_VERSION;

async fn list_dataengines() -> Result<Json<system::ListDataEngineResponse>, StatusCode> {
    let response = system::ListDataEngineResponse {
        meta: support::ResponseMetadata::success("list dataengines success"),
        engines: system::SUPPORT_DATA_ENGINES.clone(),
    };
    Ok(Json(response))
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
