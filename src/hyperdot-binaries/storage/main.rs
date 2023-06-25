// use hyperdot_node::storeage::server::Server;
// use hyperdot_node::storeage::server::ServerArgs;
use std::path::Path;

use anyhow::anyhow;
use clap::Parser;
use hyperdot_core::config::Catalog;
use hyperdot_node::storeage::Server;
use tracing_subscriber::util::SubscriberInitExt;
#[derive(Debug, Parser)]
struct AppArgs {
    /// The name of stroage node.
    #[arg(long)]
    name: String,
    /// The catalog config path.
    #[arg(long)]
    catalog: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let args = AppArgs::parse();
    tracing::info!("preapre {} storage node", args.name);
    let catalog = Catalog::try_from(Path::new(&args.catalog))
        .map_err(|err| anyhow!("init catalog error: {}", err))?;
    let node_cfg = catalog.storage.get_node_config(&args.name).map_or(
        Err(anyhow::anyhow!(
            "provide name({}) cannot found in {}",
            args.name,
            args.catalog
        )),
        |node| Ok(node),
    )?;

    let mut json_rpc_server = Server::async_new(node_cfg).await?;
    json_rpc_server.start().await?;
    json_rpc_server.stopped().await?;
    Ok(())
}
