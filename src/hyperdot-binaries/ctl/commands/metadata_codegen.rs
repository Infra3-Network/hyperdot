use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
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
            let path =
                PathBuf::from_str("./config/metadata.json").map_err(|err| anyhow!("{}", err))?;
            self.config = Some(path.as_os_str().to_str().unwrap().to_string());
        }

        if self.dir.is_none() {
            let path =
                PathBuf::from_str("./src/hyperdot-core/src").map_err(|err| anyhow!("{}", err))?;
            self.dir = Some(path.as_os_str().to_str().unwrap().to_string());
        }

        if self.mod_name.is_none() {
            self.mod_name = Some(String::from("runtime_api"))
        }

        Ok(self)
    }
}

impl MetadataCodegen {
    pub fn execute(mut self) -> anyhow::Result<()> {
        self = self.as_default()?;
        println!("{:?}", self);
        let cfgs = self.parse_config()?;
        println!("{:?}", cfgs);
        let mut tokens = vec![];
        for cfg in cfgs.iter() {
            tokens.push(Self::runtime_api_codegen(cfg)?);
        }

        self.write_tokens(&tokens)
    }

    fn parse_config(&self) -> anyhow::Result<Vec<MetadataConfig>> {
        let fs = File::open(&self.config.as_ref().unwrap())?;
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
                let output_rs = cfg.name.clone().to_lowercase();

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

    fn write_tokens(&self, tokens: &[MetadataToken]) -> anyhow::Result<()> {
        // Create the directory if it doesn't exist
        let dir_name = self.dir.as_ref().unwrap();
        let mod_name = self.mod_name.as_ref().unwrap();
        let mod_path = format!("{}/{}", dir_name, mod_name);
        let mod_path = Path::new(&mod_path);
        if !mod_path.exists() {
            std::fs::create_dir(mod_path)
                .map_err(|err| anyhow!("Failed to create directory: {}", err))?;
        }

        // Create the module file if it doesn't exist
        let mod_path = mod_path.join(format!("mod.rs"));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&mod_path)
            .map_err(|err| {
                anyhow!(
                    "Failed to open directory {}, error: {}",
                    mod_path.display(),
                    err
                )
            })?;
        self.write_tokens_to_file(file, tokens, &mod_path)
    }

    fn write_tokens_to_file(
        &self,
        mut f: std::fs::File,
        tokens: &[MetadataToken],
        mod_path: &Path,
    ) -> anyhow::Result<()> {
        for token in tokens {
            let fname = format!("{}.rs", token.output_rs);
            let fp = mod_path.join(&fname);
            let mut fs = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&fp)
                .map_err(|err| anyhow!("open mod file {} error: {}", fp.display(), err))?;

            write!(fs, "{}", token.ts).map_err(|err| {
                anyhow!("write mod file {} token error: {}", token.output_rs, err)
            })?;
        }
        for token in tokens {
            write!(f, "pub mod {};\n", token.output_rs)
                .map_err(|err| anyhow!("write mod file {} error: {}", token.output_rs, err))?;
        }

        Ok(())
    }
}
