mod asset;
mod hex;
mod naming;

use std::path::Path;

use anyhow::{Result, bail};

pub use asset::{read_compressed_program_bytes_from_asset, read_program_bytes_from_asset};
pub use hex::{
    decode_compressed_hex_text, decode_gzip_bytes, encode_compressed_hex_bytes,
    read_compressed_hex_bytes,
};
pub use naming::sanitize_output_stem;

pub fn read_program_bytes(path: &Path) -> Result<Vec<u8>> {
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "hex" => read_compressed_hex_bytes(path),
        "asset" => read_program_bytes_from_asset(path),
        _ => bail!(
            "unsupported program input extension for {}: expected .hex or .asset",
            path.display()
        ),
    }
}
