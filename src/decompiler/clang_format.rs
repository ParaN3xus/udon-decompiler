use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use serde::Deserialize;
use serde_yaml::{Mapping, Value};

use crate::decompiler::{DecompileError, Result};
use crate::str_constants::CLANG_FORMAT_ASSUME_FILENAME_CS;

const EMBEDDED_CLANG_FORMAT_STYLE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.clang-format"));

static CLANG_FORMAT_STYLE_JSON: OnceLock<String> = OnceLock::new();

pub(crate) fn format_csharp(code: &str) -> Result<String> {
    let style = embedded_clang_format_style_json()?;

    let mut child = Command::new("clang-format")
        .arg(format!(
            "--assume-filename={}",
            CLANG_FORMAT_ASSUME_FILENAME_CS
        ))
        .arg(format!("-style={style}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => {
                DecompileError::new("clang-format not found on PATH. Install it or adjust PATH.")
            }
            _ => DecompileError::new(format!("failed to start clang-format: {error}")),
        })?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(code.as_bytes()).map_err(|error| {
            DecompileError::new(format!("failed to write clang-format stdin: {error}"))
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        DecompileError::new(format!("failed to wait for clang-format: {error}"))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            "unknown error"
        } else {
            stderr.as_str()
        };
        return Err(DecompileError::new(format!(
            "clang-format failed: {detail}"
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| DecompileError::new(format!("clang-format output is not utf-8: {error}")))
}

fn embedded_clang_format_style_json() -> Result<&'static str> {
    if let Some(style) = CLANG_FORMAT_STYLE_JSON.get() {
        return Ok(style.as_str());
    }

    let style = render_embedded_clang_format_style()?;
    let style_ref = CLANG_FORMAT_STYLE_JSON.get_or_init(|| style);
    Ok(style_ref.as_str())
}

fn render_embedded_clang_format_style() -> Result<String> {
    let mut effective = Mapping::new();

    for document in serde_yaml::Deserializer::from_str(EMBEDDED_CLANG_FORMAT_STYLE) {
        let value = Value::deserialize(document).map_err(|error| {
            DecompileError::new(format!("failed to parse embedded .clang-format: {error}"))
        })?;

        let Value::Mapping(mapping) = value else {
            if matches!(value, Value::Null) {
                continue;
            }
            return Err(DecompileError::new(
                "embedded .clang-format must contain only YAML mappings",
            ));
        };

        if should_apply_clang_format_section(&mapping) {
            merge_yaml_mapping(&mut effective, &mapping);
        }
    }

    serde_json::to_string(&Value::Mapping(effective)).map_err(|error| {
        DecompileError::new(format!(
            "failed to serialize embedded clang-format style: {error}"
        ))
    })
}

fn should_apply_clang_format_section(mapping: &Mapping) -> bool {
    match mapping.get(Value::String("Language".to_string())) {
        None => true,
        Some(Value::String(language)) => language.eq_ignore_ascii_case("csharp"),
        Some(_) => false,
    }
}

fn merge_yaml_mapping(target: &mut Mapping, source: &Mapping) {
    for (key, value) in source {
        match (target.get_mut(key), value) {
            (Some(Value::Mapping(target_map)), Value::Mapping(source_map)) => {
                merge_yaml_mapping(target_map, source_map);
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}
