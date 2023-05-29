use anyhow::Result as AnyResult;
use futures::StreamExt;
use subxt::config::Header;
use subxt::Config;

use crate::jsee::JseeRpcClient;

impl<C> JseeRpcClient<C>
where C: Config
{
    async fn block_stream(&self) -> AnyResult<()> {
        let online = self.get_online();
        println!("Genesis Hash: {:?}", online.genesis_hash());
        let genesis_block = online.blocks().at(online.genesis_hash()).await?;
        println!("Genesis Block #{:?}", genesis_block.number().into());
        let mut blocks_sub = online.blocks().subscribe_finalized().await?;
        // For each block, print a bunch of information about it:
        while let Some(block) = blocks_sub.next().await {
            let block = block?;

            let block_number = block.header().number().into();
            let block_hash = block.hash();

            println!("Block #{block_number}:");
            println!("  Hash: {:?}", block_hash);
            // println!("  Extrinsics:");

            // // Log each of the extrinsic with it's associated events:
            // let body = block.body().await?;
            // for ext in body.extrinsics().into_iter(){
            //     let idx = ext.index();
            //     let events = ext.events().await?;
            //     let bytes_hex = format!("0x{}", hex::encode(ext.bytes()));

            //     // See the API docs for more ways to decode extrinsics:
            //     // let decoded_ext = ext.as_root_extrinsic::<polkadot::Call>();

            //     println!("    Extrinsic #{idx}:");
            //     println!("      Bytes: {bytes_hex}");
            //     // println!("      Decoded: {decoded_ext:?}");
            //     println!("      Events:");

            //     for evt in events.iter() {
            //         let evt = evt?;

            //         let pallet_name = evt.pallet_name();
            //         let event_name = evt.variant_name();
            //         let event_values = evt.field_values()?;

            //         println!("        {pallet_name}_{event_name}");
            //         println!("          {}", event_values);
            //     }
            // }
        }

        Ok(())
    }
}

#[tokio::test]
async fn it_works() {
    use subxt::config::PolkadotConfig;

    use crate::jsee::JseeRpcClient;
    let client = JseeRpcClient::<PolkadotConfig>::with_polkadot_testnet()
        .await
        .unwrap();
    // run it by: cargo test --package hyperdot-common-rpc --lib -- blocks::it_works --exact --nocapture
    client.block_stream().await;
}
