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
    Bytecode {},
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

fn main() -> Result<()> {
    let args = CLI::parse();

    match args.command {
        Commands::TreeWalker(args) => match args.command {
            TreeWalkerCommands::Run { script } => {
                println!("Tree-walker {}", script.display());
                Ok(())
            }
        },
        Commands::Bytecode {} => bytecode::run(),
    }
}
