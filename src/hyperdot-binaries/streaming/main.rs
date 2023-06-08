// use hyperdot_node::streaming::jsonrpc::server::JsonRpcServerParams;
use hyperdot_node::streaming::BlockStreaming;
use hyperdot_node::streaming::OpenParams;
use hyperdot_node::streaming::SpawnPolkadotParams;
use tracing_subscriber::util::SubscriberInitExt;
use subxt::PolkadotConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let filter = tracing_subscriber::EnvFilter::try_from_default_env()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

	let params = OpenParams {
		child_urls: vec![
			// scheme://host:port/
			"jsonrpc://127.0.0.1:15722?scheme=http".to_string(),
		]
	};
	let mut streaming = BlockStreaming::<PolkadotConfig>::open(&params).await?;

	let params = SpawnPolkadotParams {
		scheme: "ws".to_string(),	
		host: "192.168.124.34".to_string(),
		port: 9944,
		// block_sync_urls: vec![
		// 	"polkadot://192.168.124.34:9944?scheme=ws&block_start=1&block_end=10000".to_string(),
		// ]
	};
	let handle = streaming.spawn(&params).await?;
	handle.stopped().await?;
	Ok(())
}