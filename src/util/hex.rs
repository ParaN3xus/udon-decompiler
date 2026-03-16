use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result, bail};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

pub fn decode_gzip_bytes(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut out = Vec::<u8>::new();
    decoder
        .read_to_end(&mut out)
        .context("failed to gzip-decompress payload")?;
    Ok(out)
}

pub fn decode_compressed_hex_text(text: &str) -> Result<Vec<u8>> {
    let normalized = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    if normalized.is_empty() {
        bail!("empty hex input");
    }
    if normalized.len() % 2 != 0 {
        bail!("hex input has odd length");
    }

    let mut compressed = Vec::<u8>::with_capacity(normalized.len() / 2);
    for index in (0..normalized.len()).step_by(2) {
        let byte =
            u8::from_str_radix(&normalized[index..index + 2], 16).with_context(|| {
                format!(
                    "invalid hex byte '{}' at offset {}",
                    &normalized[index..index + 2],
                    index,
                )
            })?;
        compressed.push(byte);
    }

    decode_gzip_bytes(&compressed).context("failed to gzip-decompress hex payload")
}

pub fn read_compressed_hex_bytes(path: &Path) -> Result<Vec<u8>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    decode_compressed_hex_text(&raw).with_context(|| format!("failed to decode {}", path.display()))
}

pub fn encode_compressed_hex_bytes(bytes: &[u8]) -> Result<String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .context("failed to gzip-compress bytes")?;
    let compressed = encoder.finish().context("failed to finalize gzip stream")?;

    let mut out = String::with_capacity(compressed.len() * 2);
    for byte in compressed {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    Ok(out)
}
