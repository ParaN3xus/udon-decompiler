use std::path::Path;

use anyhow::{Context, Result, bail};
use unity_asset_yaml::python_like_api::PythonLikeUnityDocument;

use super::hex::decode_gzip_bytes;

pub fn read_compressed_program_bytes_from_asset(path: &Path) -> Result<Vec<u8>> {
    let doc = PythonLikeUnityDocument::load_yaml(path, false).map_err(|e| {
        anyhow::anyhow!("failed to load Unity asset yaml {}: {}", path.display(), e)
    })?;
    let entry = doc
        .get(
            Some("MonoBehaviour"),
            Some(&["serializedProgramCompressedBytes"]),
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "failed to find serializedProgramCompressedBytes in {}: {}",
                path.display(),
                e
            )
        })?;

    if let Some(text) = entry.get_string("serializedProgramCompressedBytes") {
        let normalized = text
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        if normalized.is_empty() {
            bail!(
                "serializedProgramCompressedBytes is empty in {}",
                path.display()
            );
        }
        let mut out = Vec::<u8>::with_capacity(normalized.len() / 2);
        if normalized.len() % 2 != 0 {
            bail!(
                "serializedProgramCompressedBytes hex has odd length in {}",
                path.display()
            );
        }
        for index in (0..normalized.len()).step_by(2) {
            let byte =
                u8::from_str_radix(&normalized[index..index + 2], 16).with_context(|| {
                    format!(
                        "invalid asset hex byte '{}' at offset {} in {}",
                        &normalized[index..index + 2],
                        index,
                        path.display()
                    )
                })?;
            out.push(byte);
        }
        return Ok(out);
    }

    if let Some(values) = entry.get_array("serializedProgramCompressedBytes") {
        let mut out = Vec::<u8>::with_capacity(values.len());
        for (index, value) in values.iter().enumerate() {
            let int = value.as_integer().ok_or_else(|| {
                anyhow::anyhow!(
                    "serializedProgramCompressedBytes[{}] is not an integer in {}",
                    index,
                    path.display()
                )
            })?;
            let byte = u8::try_from(int).map_err(|_| {
                anyhow::anyhow!(
                    "serializedProgramCompressedBytes[{}] out of byte range ({}) in {}",
                    index,
                    int,
                    path.display()
                )
            })?;
            out.push(byte);
        }
        return Ok(out);
    }

    bail!(
        "serializedProgramCompressedBytes has unsupported shape in {}",
        path.display()
    )
}

pub fn read_program_bytes_from_asset(path: &Path) -> Result<Vec<u8>> {
    let compressed = read_compressed_program_bytes_from_asset(path)?;
    decode_gzip_bytes(&compressed).with_context(|| {
        format!(
            "failed to gzip-decompress serializedProgramCompressedBytes from {}",
            path.display()
        )
    })
}
