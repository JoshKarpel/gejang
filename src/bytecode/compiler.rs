use anyhow::Result;

use crate::bytecode::scanner::scan;

pub fn compile(source: &str) -> Result<()> {
    scan(source)
}
