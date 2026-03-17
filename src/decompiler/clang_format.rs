use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use serde::Deserialize;
use serde_yaml::{Mapping, Value};

use crate::decompiler::{DecompileError, Result};
use crate::str_constants::{CLANG_FORMAT_ASSUME_FILENAME_CS, CLANG_FORMAT_PATH_ENV_VAR};

const EMBEDDED_CLANG_FORMAT_STYLE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.clang-format"));

static CLANG_FORMAT_STYLE_JSON: OnceLock<String> = OnceLock::new();

pub(crate) fn format_csharp(code: &str, override_path: Option<&Path>) -> Result<String> {
    let style = embedded_clang_format_style_json()?;
    let clang_format = resolve_clang_format_program(override_path);

    let mut child = Command::new(&clang_format)
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
                DecompileError::new("clang-format not found.".to_string())
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

fn resolve_clang_format_program(override_path: Option<&Path>) -> PathBuf {
    if let Some(override_path) = override_path {
        return override_path.to_path_buf();
    }

    if let Some(from_env) = std::env::var_os(CLANG_FORMAT_PATH_ENV_VAR)
        && !from_env.is_empty()
    {
        return PathBuf::from(from_env);
    }

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
        && let Some(local) = find_local_clang_format(exe_dir)
    {
        return local;
    }

    PathBuf::from(clang_format_program_name())
}

fn find_local_clang_format(exe_dir: &Path) -> Option<PathBuf> {
    let candidate = exe_dir.join(clang_format_program_name());
    candidate.is_file().then_some(candidate)
}

fn clang_format_program_name() -> &'static str {
    if cfg!(windows) {
        "clang-format.exe"
    } else {
        "clang-format"
    }
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
