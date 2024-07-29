use anyhow::Result;

mod bytecode;

fn main() -> Result<()> {
    bytecode::run()?;

    Ok(())
}
