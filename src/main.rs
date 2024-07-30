use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

mod bytecode;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the tree-walking interpreter
    TreeWalker(TreeWalkerArgs),
    /// Run the bytecode virtual machine interpreter
    Bytecode(ByteCodeArgs),
}

#[derive(Debug, Args)]
struct TreeWalkerArgs {
    #[command(subcommand)]
    command: TreeWalkerCommands,
}

#[derive(Debug, Subcommand)]
enum TreeWalkerCommands {
    Run { script: PathBuf },
}

#[derive(Debug, Args)]
struct ByteCodeArgs {
    #[command(subcommand)]
    command: crate::ByteCodeCommands,
}

#[derive(Debug, Subcommand)]
enum ByteCodeCommands {
    Run { script: PathBuf },
}

fn main() -> Result<()> {
    let args = CLI::parse();

    match args.command {
        Commands::TreeWalker(args) => match args.command {
            TreeWalkerCommands::Run { script } => Ok(()),
        },
        Commands::Bytecode(args) => match args.command {
            ByteCodeCommands::Run { script } => bytecode::run(),
        },
    }
}
