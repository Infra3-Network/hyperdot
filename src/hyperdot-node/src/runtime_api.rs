// use std::env;
// use std::path::Path;
// lazy_static::lazy_static! {
//     static ref POLKADOT_METADATA_SMALL_PATH: String = {
//         let manifest_dir =
//             env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable unset");
//         let metadatas_dir = Path::new(&manifest_dir).join("metadatas").join("polkadot_metadata_small.scale");
//         metadatas_dir.as_os_str().to_str().unwrap().to_string()
//     };
// }

#[subxt::subxt(runtime_metadata_path = "E:\\Workspace\\codes\\infra3\\hyperdot\\metadatas\\polkadot_metadata_full.scale")]
pub mod polkadot {}
