use std::fs;

use anyhow::Result as AnyResult;
use subxt::ext::sp_core::Decode;
use subxt::Config;

use subxt_codegen::utils::MetadataVersion;
use subxt_codegen::CratePath;
use subxt_codegen::DerivesRegistry;
use subxt_codegen::TypeSubstitutes;
use subxt_metadata::Metadata;
use super::rpc::JseeRpcClient;
use super::rpc::JseeRpcClientParams;
use super::rpc::POLKADOT_MAINNET;
use super::rpc::SUBSTRATE_LOCALNET;
// use hyperdot_common_rpc::PolkadotConfiguredClient;
use crate::storeage::PolkadotStorageChannel;
use crate::storeage::PolkadotStorageChannelParams;

#[async_trait::async_trait]
pub trait BlockIndexer<C: Config> {
    async fn sync_blocks(&self) -> AnyResult<()>;
}

#[async_trait::async_trait]
pub trait Indexer<C: Config> {
    type Block: BlockIndexer<C>;
}

pub struct IndexerImpl<C: Config>
where Self: 'static
{
    pub(crate) client: JseeRpcClient<C>,
    // pub(crate) storage_channel: PolkadotStorageChannel,
}

// pub struct PolkadotIndexer {
//     pub(crate) client: PolkadotConfiguredClient,
//     pub(crate) storage_channel: PolkadotStorageChannel,
// }

impl<C: Config> IndexerImpl<C> {
    /// Create an indexer for the test net
    pub async fn dev() -> AnyResult<Self> {
        let client =
            JseeRpcClient::<C>::async_new(SUBSTRATE_LOCALNET, &JseeRpcClientParams::default())
                .await?;
        // let client = PolkadotConfiguredClient::testnet().await?;
        // let storage_channel =
        // PolkadotStorageChannel::new(PolkadotStorageChannelParams::dev()).await?;
        // let storage = PolkadotStorage::new().await?;
        Ok(Self {
            client,
            // storage_channel,
        })
    }
}

async fn generate_runtime_api_from_url(url: &str) -> AnyResult<()> {
    let uri = url.parse()?;
    let encoded_metadata_bytes =
        subxt_codegen::utils::fetch_metadata_bytes(&uri, MetadataVersion::Latest).await?;
    let metadata = Metadata::decode(&mut &*encoded_metadata_bytes)?;
    generate_runtime_api(metadata);

    Ok(())
}

fn generate_runtime_api(metadata: Metadata) {
    let runtime_api_mod = syn::parse_quote!(
        pub mod runtime_api {}
    );

    // Default module derivatives.
    let mut derives = DerivesRegistry::with_default_derives(&CratePath::default());
    // Default type substitutes.
    let substs = TypeSubstitutes::with_default_substitutes(&CratePath::default());
    // Generate the Runtime API.
    let generator = subxt_codegen::RuntimeGenerator::new(metadata);
    // Include metadata documentation in the Runtime API.
    let generate_docs = true;
    let runtime_api = generator
        .generate_runtime(
            runtime_api_mod,
            derives,
            substs,
            CratePath::default(),
            generate_docs,
        )
        .unwrap();
    println!("{}", runtime_api);
}

#[tokio::test]
async fn it_works() {
    generate_runtime_api_from_url(SUBSTRATE_LOCALNET).await.unwrap();
}
