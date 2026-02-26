use std::path::Path;

use anyhow::{Context, Result, bail};

pub fn read_normalized_base64(path: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let normalized = raw
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    if normalized.is_empty() {
        bail!("empty b64 input: {}", path.display());
    }
    Ok(normalized)
}
