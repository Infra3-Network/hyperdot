use hyperdot_node::storeage::jsonrpc::server::JsonRpcServer;
use hyperdot_node::storeage::jsonrpc::server::JsonRpcServerParams;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let params = JsonRpcServerParams::dev();
    let server = JsonRpcServer::new(params).await?;
    let handler = server.start().await?;
    handler.stopped().await?;
    Ok(())
}
