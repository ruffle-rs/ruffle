use clap::{Parser, Subcommand};

#[derive(Parser)]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Compile {
        #[clap(value_parser)]
        out_dir: String,
    },
    Lint,
}
