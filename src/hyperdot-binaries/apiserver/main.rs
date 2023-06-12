use hyperdot_node::api::server::ApiServer;
use hyperdot_node::api::server::ApiServerParams;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let params = ApiServerParams {
        http_address: "127.0.0.1:3000".to_string(),
        polkadot_pg_client_address:
            "host=hyperdot port=5432 user=postgres password=postgres dbname = polkadot".to_string(),
    };
    let mut server = ApiServer::new(params);
    server.start_http_server().await?;
    server.stopped().await
}
