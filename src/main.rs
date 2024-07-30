use anyhow::Result;
use clap::{Parser, Subcommand};

mod bytecode;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the tree-walking interpreter
    TreeWalker {},
    /// Run the bytecode virtual machine interpreter
    Bytecode {},
}

fn main() -> Result<()> {
    let args = CLI::parse();

    match args.command {
        Commands::TreeWalker {} => {
            println!("Tree-Walker");
            Ok(())
        }
        Commands::Bytecode {} => bytecode::run(),
    }
}
