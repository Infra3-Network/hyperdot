
use super::jsonrpc::JsonRpcServer;
use super::api::ApiServer;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChainStorageArgs {
	pub chain: String,
	pub storage_urls: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerArgs {
	pub jsonrpc_server_address: String,
	pub http_server_address: String,
	pub chains: Vec<ChainStorageArgs>,
}

impl TryFrom<&str> for ServerArgs {
	type Error = anyhow::Error;
	fn try_from(path: &str) -> Result<Self, Self::Error> {
		let file = std::fs::File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let args = serde_json::from_reader(reader)?;
		Ok(args)
	}
}

pub struct Server {
	jsonrpc_server: JsonRpcServer,
	api_server: ApiServer,	
}

impl Server {
	pub async fn new(args: ServerArgs) -> anyhow::Result<Self> {
		let jsonrpc_server = JsonRpcServer::new(args.clone()).await?;
		let api_server = ApiServer::new(args.clone()).await?;
		Ok(Self {
			jsonrpc_server,
			api_server,
		})
	}

	pub async fn start(&mut self) -> anyhow::Result<()> {
		self.jsonrpc_server.start().await?;
		self.api_server.start().await?;
		Ok(())
	}

	pub async fn stopped(self) -> anyhow::Result<()> {
		self.jsonrpc_server.stopped().await?;
		self.api_server.stopped().await?;
		Ok(())
	}
}