use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use quote::quote;
use serde::ser::SerializeStruct;
use serde::Deserializer;
use subxt_codegen::utils::Uri;
use subxt_codegen::CodegenError;
use subxt_codegen::CratePath;
use subxt_codegen::DerivesRegistry;
use subxt_codegen::RuntimeGenerator;
use subxt_codegen::TypeSubstitutes;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MetadataConfig {
    name: String,
    network: Option<String>,
    path: Option<String>,
    generate: bool,
}

const METADATA_JSON: &str = include_str!("metadata.json");

lazy_static::lazy_static! {
    static ref META_CONFIG: Vec<MetadataConfig> = serde_json::from_str(&METADATA_JSON).unwrap();
}

fn main() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable unset");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    for cfg in META_CONFIG.iter() {
        runtime_api_codegen(cfg)
    }
}

fn runtime_api_codegen(cfg: &MetadataConfig) {
    // let runtime_api_name = syn::parse_str::<syn::LitStr>(&format!("\"{}\"", cfg.name)).unwrap();
    let runtime_api_name_ident = quote::format_ident!("{}", cfg.name);
    let runtime_item_mod = syn::parse_quote!(
        pub mod #runtime_api_name_ident {}
    );

    println!("runtime_item_mod = {:#?}", runtime_item_mod);

    let mut derives = DerivesRegistry::with_default_derives(&CratePath::default());
    let substs = TypeSubstitutes::with_default_substitutes(&CratePath::default());
    let generate_docs = true;

    match (cfg.network.as_ref(), cfg.path.as_ref()) {
        (None, Some(path)) => todo!(),
        (Some(url), None) => {
            let url = Uri::from_str(&url)
                .unwrap_or_else(|_| panic!("Cannot download metadata; invalid url: {}", url));

            let runtime_api = subxt_codegen::generate_runtime_api_from_url(
                runtime_item_mod,
                &url,
                derives,
                substs,
                CratePath::default(),
                generate_docs,
                false,
            )
            .unwrap();

            let mut fs = std::fs::File::create(format!("{}.rs", cfg.name.to_lowercase())).unwrap();
            write!(fs, "{}", runtime_api).unwrap();
        }
        (Some(url), Some(path)) => {
            if !path.is_empty() {
                todo!()
            }

            if !url.is_empty() {
                todo!()
            }

            panic!("invalid config")
        }
        (None, None) => panic!("invalid config"),
    }
}
