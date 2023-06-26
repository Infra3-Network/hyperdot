use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use proc_macro2::TokenStream;
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

struct MetadataToken {
    ts: TokenStream,
    output_rs: String,
}

// const METADATA_JSON: &str = include_str!("metadata.json");

// lazy_static::lazy_static! {
//     static ref META_CONFIG: Vec<MetadataConfig> = serde_json::from_str(&METADATA_JSON).unwrap();
// }

#[derive(Debug, clap::Parser)]
pub struct MetadataCodegen {
    #[clap(long)]
    config: Option<String>,
    #[clap(long)]
    dir: Option<String>,
    #[clap(long)]
    mod_name: Option<String>,
}

impl MetadataCodegen {
    pub fn as_default(mut self) -> anyhow::Result<Self> {
        if self.config.is_none() {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR")
                .map_err(|_| anyhow::anyhow!("CARGO_MANIFEST_DIR env variable unset"))?;
            let path = PathBuf::from(manifest_dir).join("config/metdata.json");
        }
        self
    }
}

impl MetadataCodegen {
    pub fn execute(self) -> anyhow::Result<()> {
        let cfgs = self.parse_config()?;
        let mut tokens = vec![];
        for cfg in cfgs.iter() {
            tokens.push(Self::runtime_api_codegen(cfg)?);
        }
        Ok(())
    }

    fn parse_config(&self) -> anyhow::Result<Vec<MetadataConfig>> {
        let fs = File::open(&self.config)?;
        let rd = BufReader::new(fs);
        serde_json::from_reader(rd).map_err(|err| anyhow::anyhow!("{}", err))
    }

    fn runtime_api_codegen(cfg: &MetadataConfig) -> anyhow::Result<MetadataToken> {
        let runtime_api_name_ident = quote::format_ident!("{}", cfg.name);
        let runtime_item_mod = syn::parse_quote!(
            pub mod #runtime_api_name_ident {}
        );

        let derives = DerivesRegistry::with_default_derives(&CratePath::default());
        let substs = TypeSubstitutes::with_default_substitutes(&CratePath::default());
        let generate_docs = true;

        match (cfg.network.as_ref(), cfg.path.as_ref()) {
            (None, Some(path)) => todo!(),
            (Some(url), None) => {
                let url = Uri::from_str(&url)
                    .unwrap_or_else(|_| panic!("Cannot download metadata; invalid url: {}", url));

                let ts = subxt_codegen::generate_runtime_api_from_url(
                    runtime_item_mod,
                    &url,
                    derives,
                    substs,
                    CratePath::default(),
                    generate_docs,
                    false,
                )
                .map_err(|err| anyhow::anyhow!("{}", err))?;
                let output_rs = cfg.name.clone();

                Ok(MetadataToken { ts, output_rs })
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

    fn write_tokens(tokens: &[MetadataToken]) -> anyhow::Result<()> {}
}
