use anyhow::Result as AnyResult;
use futures::StreamExt;
use hyperdot_common_types::WriteBlockHeaderRequest;
use subxt::config::Header;

use crate::indexer::BlockIndexer;
use crate::indexer::PolkadotIndexer;
use crate::storeage::StorageChannel;

#[async_trait::async_trait]
impl BlockIndexer for PolkadotIndexer {
    async fn sync_blocks(&self) -> AnyResult<()> {
        let online = self.client.get_online();
        let _ = tokio::spawn(async move {
            let genesis_block = online.blocks().at(online.genesis_hash()).await.unwrap();
            let finalized_block_hash = online.rpc().finalized_head().await.unwrap();
            let finlized_block = online
                .rpc()
                .block(Some(finalized_block_hash))
                .await
                .unwrap()
                .unwrap()
                .block;
            println!(
                "Start sync gap {} -> {}",
                genesis_block.number(),
                finlized_block.header.number()
            );
            println!("Genesis Block #{}", genesis_block.number());
            println!("Genesis Hash: {:?}", online.genesis_hash());

            // for block_number in genesis_block.number()..=finlized_block.header.number() {
            //     let block_hash = online.rpc().block_hash(block_number).await.unwrap();
            //     // println!("Block #{}:", block_number);
            //     // println!("  Hash: {:?}", block_hash);
            // }
        });

        let online = self.client.get_online();
        println!("Genesis Hash: {:?}", online.genesis_hash());
        let genesis_block = online.blocks().at(online.genesis_hash()).await?;
        let mut blocks_sub = online.blocks().subscribe_finalized().await?;
        // For each block, print a bunch of information about it:
        while let Some(block) = blocks_sub.next().await {
            let block = block?;
            let block_header = block.header();
            let block_hash = block.hash();
            let req = WriteBlockHeaderRequest {
                block_number: block_header.number() as u64,
                block_hash: block_hash.as_bytes().to_vec(),
                parent_hash: block_header.parent_hash.as_bytes().to_vec(),
                state_root: block_header.state_root.as_bytes().to_vec(),
                extrinsics_root: block_header.extrinsics_root.as_bytes().to_vec(),
            };
            let _ = self.storage_channel.write_block(req).await?;

            // println!("Block #{block_number}:");
            // println!("  Hash: {:?}", block_hash);
            // println!("  Extrinsics:");

            // Log each of the extrinsic with it's associated events:
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
    let index = PolkadotIndexer::testnet().await.unwrap();
    index.sync_blocks().await.unwrap();
}
