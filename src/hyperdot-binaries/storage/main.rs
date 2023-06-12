use hyperdot_node::storeage::server::JsonRpcServer;
use hyperdot_node::storeage::server::JsonRpcServerParams;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let params = JsonRpcServerParams {
        address: "127.0.0.1:15722".to_string(),
        chain: "polkadot".to_string(),
        storages: vec![
            "postgres://hyperdot:5432?user=postgres&password=postgres&dbname=polkadot".to_string(),
        ],
    };
    let server = JsonRpcServer::new(params).await?;
    let handler = server.start().await?;
    handler.stopped().await?;
    Ok(())
}
