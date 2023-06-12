use std::sync::Arc;

use tokio::task::JoinHandle;

use super::handle::Context;
use super::route;

pub struct ApiServerParams {
    /// The http server listend at.
    pub http_address: String,

    /// TODO: so suck.
    pub polkadot_pg_client_address: String,
}
pub struct ApiServer {
    params: ApiServerParams,
    http_serv_handle: Option<JoinHandle<anyhow::Result<()>>>,
}

impl ApiServer {
    pub fn new(params: ApiServerParams) -> Self {
        Self {
            params,
            http_serv_handle: None,
        }
    }

    pub async fn start_http_server(&mut self) -> anyhow::Result<()> {
        if self.http_serv_handle.is_some() {
            return Err(anyhow::anyhow!(
                "🙅 the http server not empty, it's already running?"
            ));
        }

        let (polkadot_pg_client, connection) = tokio_postgres::connect(
            &self.params.polkadot_pg_client_address,
            tokio_postgres::NoTls,
        )
        .await?;

        // The connection object performs the actual communication with the database,
        // // so spawn it off to run on its own.
        let _pg_conn_handle = tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::error!("🐛 PostgresStorage: postgres connection error: {}", err);
                return Err(anyhow::anyhow!("{}", err));
            }
            return Ok(());
        });

        let ctx = Context {
            polkadot_pg_client: Arc::new(polkadot_pg_client),
        };

        let app = route::build(ctx);

        let addr = self.params.http_address.as_str().parse()?;

        let handle = tokio::spawn(async move {
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .map_err(|err| anyhow::anyhow!("{}", err))
        });

        tracing::info!(
            "🏃 http apiserver has been listend at {}",
            self.params.http_address,
        );
        self.http_serv_handle = Some(handle);

        Ok(())
    }

    pub async fn stopped(self) -> anyhow::Result<()> {
        if let Some(serv) = self.http_serv_handle {
            serv.await?;
        }

        Ok(())
    }
}
