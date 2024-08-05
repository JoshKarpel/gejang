use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

mod bytecode;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the tree-walking interpreter
    #[command(alias = "tw")]
    TreeWalker(TreeWalkerArgs),
    /// Run the bytecode virtual machine interpreter
    #[command(alias = "bc")]
    #[command(alias = "vm")]
    Bytecode(ByteCodeArgs),
}

#[derive(Debug, Args)]
struct TreeWalkerArgs {
    #[command(subcommand)]
    command: TreeWalkerCommands,
}

#[derive(Debug, Subcommand)]
enum TreeWalkerCommands {
    /// Execute a script.
    Run { script: PathBuf },
}

#[derive(Debug, Args)]
struct ByteCodeArgs {
    #[command(subcommand)]
    command: ByteCodeCommands,
}

#[derive(Debug, Subcommand)]
enum ByteCodeCommands {
    /// Execute a script.
    Run { script: Option<PathBuf> },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::TreeWalker(args) => match args.command {
            TreeWalkerCommands::Run { script: _ } => Ok(()),
        },
        Commands::Bytecode(args) => match args.command {
            ByteCodeCommands::Run { script: s } => {
                if let Some(path) = s {
                    bytecode::run(&path)
                } else {
                    bytecode::repl()
                }
            }
        },
    }
}
