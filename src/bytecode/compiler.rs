use crate::shared::scanner::scan;
use anyhow::Result;
use itertools::Itertools;

pub fn compile(source: &str) -> Result<()> {
    let _ = scan(source).collect_vec();
    Ok(())
}
