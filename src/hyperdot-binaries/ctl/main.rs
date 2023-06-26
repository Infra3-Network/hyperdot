use anyhow::anyhow;
use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use commands::MetadataCodegen;

mod commands;

#[derive(Debug, Parser)]
pub struct Args {
    /// Print version info and exit.
    #[clap(short = 'V', long)]
    version: bool,
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Parser, Debug)]
enum Cmd {
    /// Generate runtime metadata
    #[clap(name = "metadata-codegen")]
    MetadataCodegen(MetadataCodegen),
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;
    match args.cmd {
        Some(Cmd::MetadataCodegen(cmd)) => cmd.execute(),
        None => {
            Args::command().print_long_help()?;
            // Note: clap uses an exit code of 2 when CLI parsing fails
            std::process::exit(2);
        }
    }
}
