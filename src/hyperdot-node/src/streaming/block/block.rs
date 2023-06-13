
use anyhow::Result as AnyResult;
use subxt::blocks::ExtrinsicEvents;
use subxt::config::Header;
use subxt::events::Phase;
use subxt::Config;

use crate::runtime_api::polkadot::Event;
// use crate::types::EventPhase;

// #[derive(Default)]
// struct ParsedEventPair {
//     raw_events: Vec<Event>,
//     decoded_events: Vec<EventDecode>,
// }

// impl ParsedEventPair {
//     fn fill<C: Config>(header: &C::Header, events: &ExtrinsicEvents<C>) -> AnyResult<Self> {
//         let block_hash = header.hash();
//         let block_number = header.number().try_into()?;
//         let extrinsic_hash = events.extrinsic_hash();
//         let block_events = events.all_events_in_block();

//         println!("Extrisic {:?}", extrinsic_hash);
//         let mut event_pair = ParsedEventPair::default();
//         for block_event in block_events.iter() {
//             let block_event = block_event?;
//             let ev_index = block_event.index();
//             let ev_data = block_event.bytes().to_vec();
//             let ev_phase = match block_event.phase() {
//                 Phase::Initialization => EventPhase::Initialization,
//                 Phase::ApplyExtrinsic(val) => EventPhase::ApplyExtrinsic(val),
//                 Phase::Finalization => EventPhase::Finalization,
//             };
//             let ev_pallet_index = block_event.pallet_index();
//             let ev_pallet_name = block_event.pallet_name().to_string();

//             let topics = block_event.topics();
//             let (topic0, topic1, topic2, topic3, topic4) = match topics.len() {
//                 0 => (vec![], vec![], vec![], vec![], vec![]),
//                 1 => (topics[0].as_ref().to_vec(), vec![], vec![], vec![], vec![]),
//                 2 => (
//                     topics[0].as_ref().to_vec(),
//                     topics[1].as_ref().to_vec(),
//                     vec![],
//                     vec![],
//                     vec![],
//                 ),
//                 3 => (
//                     topics[0].as_ref().to_vec(),
//                     topics[1].as_ref().to_vec(),
//                     topics[2].as_ref().to_vec(),
//                     vec![],
//                     vec![],
//                 ),

//                 4 => (
//                     topics[0].as_ref().to_vec(),
//                     topics[1].as_ref().to_vec(),
//                     topics[2].as_ref().to_vec(),
//                     topics[3].as_ref().to_vec(),
//                     vec![],
//                 ),
//                 5 | _ => (
//                     topics[0].as_ref().to_vec(),
//                     topics[1].as_ref().to_vec(),
//                     topics[2].as_ref().to_vec(),
//                     topics[3].as_ref().to_vec(),
//                     topics[4].as_ref().to_vec(),
//                 ),
//             };
//             let raw_event = Event {
//                 block_hash: block_hash.as_ref().to_vec(),
//                 block_number,
//                 block_time: 0,
//                 extrinsic_hash: extrinsic_hash.as_ref().to_vec(),
//                 data: ev_data,
//                 index: ev_index,
//                 topic0,
//                 topic1,
//                 topic2,
//                 topic3,
//                 topic4,
//                 phase: ev_phase.clone(),
//             };

//             let decode_event = EventDecode {
//                 block_hash: block_hash.as_ref().to_vec(),
//                 block_number,
//                 block_time: 0,
//                 extrinsic_hash: extrinsic_hash.as_ref().to_vec(),
//                 index: ev_index,
//                 pallet_index: ev_pallet_index,
//                 pallet_name: ev_pallet_name,
//                 phase: ev_phase,
//                 // TODO signature
//             };

//             event_pair.raw_events.push(raw_event);
//             event_pair.decoded_events.push(decode_event);
//         }

//         Ok(event_pair)
//     }
// }


// #[tokio::test]
// async fn it_works() {
//     let url = "ws://192.168.124.34:9944";
//     let client = JseeRpcClient::<PolkadotConfig>::async_new(url, &JseeRpcClientParams::default()).await.unwrap();
//     let (tx, rx) = unbounded_channel();

//     let mut sync = Syncer::new(client);
//     sync.start(tx).unwrap();

//     let mut streaming = Streaming::new();
//     streaming.start(rx);

//     tokio::time::sleep(std::time::Duration::from_secs(600)).await;
//     // let index = IndexerImpl::<PolkadotConfig>::dev().await.unwrap();
//     // index.sync_blocks().await.unwrap();
// }
