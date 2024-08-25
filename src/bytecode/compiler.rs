use anyhow::Result;
use itertools::Itertools;

use crate::shared::scanner::scan;

pub fn compile(source: &str) -> Result<()> {
    let _ = scan(source).collect_vec();
    Ok(())
}
