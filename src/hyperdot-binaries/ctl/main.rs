use anyhow::anyhow;
use anyhow::Result;
use clap::ErrorKind;
use commands::MetadataCodegen;

mod commands;

#[derive(clap::Parser)]
#[cfg_attr(
    not(feature = "headless"),
    clap(
        name = "hyperctl",
        about = "WebAssembly standalone runtime.",
        version,
        author
    )
)]
#[cfg_attr(
    feature = "headless",
    clap(
        name = "wasmer-headless",
        about = "WebAssembly standalone runtime (headless).",
        version,
        author
    )
)]

enum HyperCtlOptions {
    /// Generate runtime metadata
    #[clap(name = "metadata-codegen")]
    MetadataCodegen(MetadataCodegen),
}
