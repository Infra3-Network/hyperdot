// use hyperdot_node::storeage::server::Server;
// use hyperdot_node::storeage::server::ServerArgs;
use std::path::Path;

use clap::Parser;
use hyperdot_common_config::Catalog;
use hyperdot_node::storeage::server::JsonRpcServer;
use tracing_subscriber::util::SubscriberInitExt;
#[derive(Debug, Parser)]
struct AppArgs {
    /// The name of stroage node.
    #[arg(long)]
    name: String,
    /// The catalog config path.
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
    let catalog = Catalog::try_from(Path::new(&args.catalog))?;
    let node_cfg = catalog.storage.get_node_config(&args.name).map_or(
        Err(anyhow::anyhow!(
            "provide name({}) cannot found in {}",
            args.name,
            args.catalog
        )),
        |node| Ok(node),
    )?;

    let mut json_rpc_server = JsonRpcServer::async_new(node_cfg).await?;
    json_rpc_server.start().await?;
    json_rpc_server.stopped().await?;
    // let args = ServerArgs::try_from(".local/storage-args.json")?;
    // let mut server = Server::new(args).await?;
    // server.start().await?;
    // server.stopped().await?;
    Ok(())
}
