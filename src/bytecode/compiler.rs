use crate::bytecode::scanner::scan;
use anyhow::Result;

pub fn compile(source: &String) -> Result<()> {
    scan(source)
}
