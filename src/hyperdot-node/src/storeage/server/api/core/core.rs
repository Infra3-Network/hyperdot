use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use axum::Router;

use super::model::core::ListDataEngine;
use super::model::core::SUPPORT_DATA_ENGINES;
use super::route::Context;
use crate::storeage::server::api::API_ROOT_PATH;

pub struct CoreHandle;

impl CoreHandle {
    async fn list_data_engine() -> Result<Json<ListDataEngine>, (StatusCode, String)> {
        Ok(Json(SUPPORT_DATA_ENGINES.clone()))
    }
}

pub struct CoreRouteBuild {
    path: String,
}

impl CoreRouteBuild {
    pub fn new() -> Self {
        Self {
            path: "core".to_string(),
        }
    }

    pub fn build(self, mut router: Router<Context>) -> anyhow::Result<Router<Context>> {
        let base = self.base_path();
        let list_dataengins = format!("{}/dataengines", base);
        tracing::info!("register {}", list_dataengins);
        router = router.route(list_dataengins.as_str(), get(CoreHandle::list_data_engine));

        Ok(router)
    }

    fn base_path(&self) -> String {
        // /apis/core
        format!("{}/{}", API_ROOT_PATH, self.path)
    }
}
