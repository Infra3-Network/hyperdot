use hyperdot_node::storeage::server::Server;
use hyperdot_node::storeage::server::ServerArgs;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let args = ServerArgs::try_from(".local/storage-args.json")?;
    let mut server = Server::new(args).await?;
    server.start().await?;
    server.stopped().await?;
    Ok(())
}
