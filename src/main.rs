use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use udon_decompiler::odin::{SymbolSection, UdonProgramBinary};
use udon_decompiler::udon_asm::{assemble_b64_with_original, disassemble_program_to_text};

#[derive(Parser, Debug)]
#[command(name = "udon-decompiler")]
#[command(about = "Udon Decompiler CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Decompile b64 to source code (placeholder for now).
    Dc {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    /// Disassemble b64 to asm text.
    Dasm {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    /// Assemble asm text back to b64.
    Asm {
        input: PathBuf,
        output: Option<PathBuf>,
        #[arg(long)]
        template: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Dc,
    Dasm,
    Asm,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Dc { input, output } => run(Mode::Dc, &input, output.as_deref(), None),
        Commands::Dasm { input, output } => run(Mode::Dasm, &input, output.as_deref(), None),
        Commands::Asm {
            input,
            output,
            template,
        } => run(Mode::Asm, &input, output.as_deref(), template.as_deref()),
    }
}

fn run(mode: Mode, input: &Path, output: Option<&Path>, template: Option<&Path>) -> Result<()> {
    if !input.exists() {
        bail!("input path does not exist: {}", input.display());
    }

    if mode != Mode::Asm && template.is_some() {
        bail!("--template is only valid for asm command");
    }

    if input.is_file() {
        process_single_file(mode, input, output, template)?;
        return Ok(());
    }

    if !input.is_dir() {
        bail!(
            "input path is neither file nor directory: {}",
            input.display()
        );
    }

    process_directory(mode, input, output, template)
}

fn process_single_file(
    mode: Mode,
    input_file: &Path,
    output: Option<&Path>,
    template: Option<&Path>,
) -> Result<()> {
    ensure_input_extension(mode, input_file)?;
    validate_template_kind_for_single(mode, template)?;

    let default_filename = match mode {
        Mode::Dc => {
            let stem = infer_class_name_from_b64_file(input_file)?;
            format!("{stem}.cs")
        }
        Mode::Dasm => {
            let stem = infer_class_name_from_b64_file(input_file)?;
            format!("{stem}.asm")
        }
        Mode::Asm => format!("{}.b64", input_file_stem(input_file)),
    };

    let output_file = match output {
        None => input_file
            .parent()
            .unwrap_or(Path::new("."))
            .join(default_filename),
        Some(path) if path.is_dir() => path.join(default_filename),
        Some(path) => path.to_path_buf(),
    };

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    process_one(mode, input_file, &output_file, template)?;
    println!("{} -> {}", input_file.display(), output_file.display());
    Ok(())
}

fn process_directory(
    mode: Mode,
    input_dir: &Path,
    output: Option<&Path>,
    template: Option<&Path>,
) -> Result<()> {
    validate_template_kind_for_directory(mode, template)?;

    let output_dir = match output {
        Some(path) => path.to_path_buf(),
        None => input_dir.with_file_name(format!("{}-decompiled", input_file_stem(input_dir))),
    };

    if output_dir.exists() && output_dir.is_file() {
        bail!(
            "output path must be a directory for directory input: {}",
            output_dir.display()
        );
    }
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output directory {}", output_dir.display()))?;

    let mut input_files = collect_input_files(input_dir, mode)?;
    if input_files.is_empty() {
        bail!(
            "no input files found for '{}' under {}",
            mode_input_glob_hint(mode),
            input_dir.display()
        );
    }
    input_files.sort();

    let mut used_names = HashMap::<String, usize>::new();
    for input_file in input_files {
        let output_file = match mode {
            Mode::Dc => {
                let base = infer_class_name_from_b64_file(&input_file)?;
                output_dir.join(unique_filename(&mut used_names, &base, "cs"))
            }
            Mode::Dasm => {
                let base = infer_class_name_from_b64_file(&input_file)?;
                output_dir.join(unique_filename(&mut used_names, &base, "asm"))
            }
            Mode::Asm => {
                let base = input_file_stem(&input_file);
                output_dir.join(unique_filename(&mut used_names, &base, "b64"))
            }
        };

        let file_template = match mode {
            Mode::Asm => template,
            _ => None,
        };
        process_one(mode, &input_file, &output_file, file_template)?;
        println!("{} -> {}", input_file.display(), output_file.display());
    }

    Ok(())
}

fn process_one(
    mode: Mode,
    input_file: &Path,
    output_file: &Path,
    template: Option<&Path>,
) -> Result<()> {
    match mode {
        Mode::Dc => {
            fs::write(output_file, "")
                .with_context(|| format!("failed to write {}", output_file.display()))?;
        }
        Mode::Dasm => {
            let b64 = read_normalized_base64(input_file)?;
            let program = UdonProgramBinary::parse_base64(&b64)
                .with_context(|| format!("failed to parse b64 from {}", input_file.display()))?;
            let asm = disassemble_program_to_text(&program).with_context(|| {
                format!(
                    "failed to disassemble program from {}",
                    input_file.display()
                )
            })?;
            let source_name = input_file
                .file_name()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or_else(|| input_file.display().to_string());
            let asm_with_source = format!("; source-b64: {}\n{}", source_name, asm);
            fs::write(output_file, asm_with_source)
                .with_context(|| format!("failed to write {}", output_file.display()))?;
        }
        Mode::Asm => {
            let asm_text = fs::read_to_string(input_file)
                .with_context(|| format!("failed to read {}", input_file.display()))?;
            let template_path =
                choose_b64_template_path(input_file, output_file, template, &asm_text)?;
            let original_b64 = read_normalized_base64(&template_path)?;
            let assembled_b64 =
                assemble_b64_with_original(&original_b64, &asm_text).with_context(|| {
                    format!(
                        "failed to assemble {} using template {}",
                        input_file.display(),
                        template_path.display()
                    )
                })?;
            fs::write(output_file, assembled_b64)
                .with_context(|| format!("failed to write {}", output_file.display()))?;
        }
    }
    Ok(())
}

fn validate_template_kind_for_single(mode: Mode, template: Option<&Path>) -> Result<()> {
    if mode != Mode::Asm {
        return Ok(());
    }
    if let Some(path) = template
        && path.exists()
        && path.is_dir()
    {
        bail!(
            "--template for single asm input must be a file path, got directory: {}",
            path.display()
        );
    }
    Ok(())
}

fn validate_template_kind_for_directory(mode: Mode, template: Option<&Path>) -> Result<()> {
    if mode != Mode::Asm {
        return Ok(());
    }
    if let Some(path) = template
        && path.exists()
        && path.is_file()
    {
        bail!(
            "--template for directory asm input must be a directory path, got file: {}",
            path.display()
        );
    }
    Ok(())
}

fn choose_b64_template_path(
    input_asm: &Path,
    output_b64: &Path,
    explicit_template: Option<&Path>,
    asm_text: &str,
) -> Result<PathBuf> {
    if let Some(template) = explicit_template {
        if template.exists() && template.is_file() {
            return Ok(template.to_path_buf());
        }
        if template.exists() && template.is_dir() {
            if let Some(hint_name) = extract_source_b64_hint(asm_text) {
                let hinted = template.join(hint_name);
                if hinted.exists() && hinted.is_file() {
                    return Ok(hinted);
                }
            }
            let by_stem = template.join(format!("{}.b64", input_file_stem(input_asm)));
            if by_stem.exists() && by_stem.is_file() {
                return Ok(by_stem);
            }
            bail!(
                "template directory does not contain matching b64 for {} (tried hint and stem)",
                input_asm.display()
            );
        }
        bail!("template path does not exist: {}", template.display());
    }

    if output_b64.exists() && output_b64.is_file() {
        return Ok(output_b64.to_path_buf());
    }
    if let Some(hint_name) = extract_source_b64_hint(asm_text) {
        let hinted = input_asm.parent().unwrap_or(Path::new(".")).join(hint_name);
        if hinted.exists() && hinted.is_file() {
            return Ok(hinted);
        }
    }
    let sibling = input_asm.with_extension("b64");
    if sibling.exists() && sibling.is_file() {
        return Ok(sibling);
    }
    bail!(
        "asm requires a b64 template file. provide --template, or ensure existing output/sibling b64 is present"
    );
}

fn extract_source_b64_hint(asm_text: &str) -> Option<String> {
    let first_line = asm_text.lines().next()?.trim();
    let prefix = "; source-b64:";
    if !first_line.starts_with(prefix) {
        return None;
    }
    let value = first_line[prefix.len()..].trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_string())
}

fn ensure_input_extension(mode: Mode, input_file: &Path) -> Result<()> {
    let ext = input_file
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let ok = match mode {
        Mode::Dc | Mode::Dasm => ext == "b64",
        Mode::Asm => ext == "asm",
    };
    if ok {
        return Ok(());
    }
    bail!(
        "unexpected input extension for {:?}: {} (expected {})",
        mode,
        input_file.display(),
        mode_input_glob_hint(mode)
    )
}

fn collect_input_files(input_dir: &Path, mode: Mode) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(input_dir)
        .with_context(|| format!("failed to read input dir {}", input_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let matched = match mode {
            Mode::Dc | Mode::Dasm => ext == "b64",
            Mode::Asm => ext == "asm",
        };
        if matched {
            out.push(path);
        }
    }
    Ok(out)
}

fn read_normalized_base64(path: &Path) -> Result<String> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let normalized = raw
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    if normalized.is_empty() {
        bail!("empty b64 input: {}", path.display());
    }
    Ok(normalized)
}

fn infer_class_name_from_b64_file(path: &Path) -> Result<String> {
    let fallback = sanitize_output_stem(input_file_stem(path));
    let b64 = read_normalized_base64(path)?;
    let program = match UdonProgramBinary::parse_base64(&b64) {
        Ok(v) => v,
        Err(_) => return Ok(fallback),
    };
    let inferred = infer_candidate_name_from_program(&program).unwrap_or(fallback);
    Ok(sanitize_output_stem(&inferred))
}

fn infer_candidate_name_from_program(program: &UdonProgramBinary) -> Option<String> {
    let entry_count = program.symbols_len(SymbolSection::EntryPoints).ok()?;
    for i in 0..entry_count {
        let entry = program.symbol_item(SymbolSection::EntryPoints, i).ok()?;
        let name = entry.name.trim();
        if name.is_empty() || name.eq_ignore_ascii_case("_start") || name.starts_with("__") {
            continue;
        }
        return Some(name.to_string());
    }
    None
}

fn unique_filename(used_names: &mut HashMap<String, usize>, base: &str, ext: &str) -> String {
    let normalized_base = sanitize_output_stem(base);
    let key = normalized_base.to_ascii_lowercase();
    let counter = used_names.entry(key).or_insert(0);
    *counter += 1;
    if *counter == 1 {
        format!("{normalized_base}.{ext}")
    } else {
        format!("{normalized_base}_{}.{ext}", *counter)
    }
}

fn sanitize_output_stem(name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let out = out.trim_matches('_');
    if out.is_empty() {
        "output".to_string()
    } else {
        out.to_string()
    }
}

fn input_file_stem(path: &Path) -> String {
    if let Some(stem) = path.file_stem().and_then(|x| x.to_str()) {
        return stem.to_string();
    }
    if let Some(name) = path.file_name().and_then(|x| x.to_str()) {
        return name.to_string();
    }
    "input".to_string()
}

fn mode_input_glob_hint(mode: Mode) -> &'static str {
    match mode {
        Mode::Dc | Mode::Dasm => "*.b64",
        Mode::Asm => "*.asm",
    }
}
