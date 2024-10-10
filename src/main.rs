#![feature(test)]
#![feature(iterator_try_collect)]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod bytecode;
mod shared;
mod walker;

#[derive(Parser, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Commands {
    /// Run the tree-walking interpreter
    #[command(alias = "tw")]
    TreeWalker(TreeWalkerArgs),
    /// Run the bytecode virtual machine interpreter
    #[command(alias = "bc")]
    #[command(alias = "vm")]
    Bytecode(ByteCodeArgs),
}

#[derive(Args, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct TreeWalkerArgs {
    #[command(subcommand)]
    command: TreeWalkerCommands,
}

#[derive(Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TreeWalkerCommands {
    /// Execute a script.
    Run { script: Option<PathBuf> },
    /// Execute a script passed directly as a string.
    Exec { script: String },
}

#[derive(Args, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct ByteCodeArgs {
    #[command(subcommand)]
    command: ByteCodeCommands,
}

#[derive(Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ByteCodeCommands {
    /// Execute a script.
    Run { script: Option<PathBuf> },
    /// Execute a script passed directly as a string.
    Exec { script: String },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::TreeWalker(args) => match args.command {
            TreeWalkerCommands::Run { script: s } => {
                if let Some(path) = s {
                    walker::exec(&std::fs::read_to_string(&path)?)
                } else {
                    walker::repl()
                }
            }
            TreeWalkerCommands::Exec { script: s } => walker::exec(&s),
        },
        Commands::Bytecode(args) => match args.command {
            ByteCodeCommands::Run { script: s } => {
                if let Some(path) = s {
                    bytecode::exec(&std::fs::read_to_string(&path)?)
                } else {
                    bytecode::repl()
                }
            }
            ByteCodeCommands::Exec { script: s } => bytecode::exec(&s),
        },
    }
}
