use crate::bytecode::scanner::scan;
use anyhow::Result;

pub fn compile(source: &str) -> Result<()> {
    scan(source)
}
