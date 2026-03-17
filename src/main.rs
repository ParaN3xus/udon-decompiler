use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use tracing::{debug, info};
use udon_decompiler::decompiler::{DecompileContext, UdonModuleInfo};
use udon_decompiler::logging::init_logging;
use udon_decompiler::odin::UdonProgramBinary;
use udon_decompiler::str_constants::{
    EXT_ASM, EXT_ASSET, EXT_CS, EXT_HEX, FILE_UDON_MODULE_INFO_JSON, INPUT_GLOB_ASM,
};
use udon_decompiler::udon_asm::{
    AsmBindAnalysis, AsmInstructionComment, assemble_hex_with_original, collect_asm_bind_analysis,
    collect_asm_instruction_comments, disassemble_program_to_text,
};
use udon_decompiler::util::{read_compressed_program_bytes_from_asset, read_program_bytes};

#[derive(Parser, Debug)]
#[command(name = "udon-decompiler")]
#[command(about = "Udon Decompiler CLI")]
#[command(version)]
struct Cli {
    #[arg(long, global = true, default_value = "info")]
    log_level: String,
    #[arg(long, global = true)]
    module_info: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Decompile .hex or .asset to C#.
    Dc {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    /// Disassemble .hex or .asset to asm text.
    Dasm {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    /// Assemble asm text back to compressed .hex.
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

enum PreparedSingleInput {
    Dc {
        ctx: Box<DecompileContext>,
    },
    Dasm {
        program: UdonProgramBinary,
        bind_analysis: AsmBindAnalysis,
        instruction_comments: Vec<AsmInstructionComment>,
        output_stem: String,
    },
    Asm {
        asm_text: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(&cli.log_level)
        .map_err(|e| anyhow::anyhow!("failed to initialize logging: {}", e))?;
    let module_info_path = cli
        .module_info
        .clone()
        .unwrap_or_else(|| PathBuf::from(FILE_UDON_MODULE_INFO_JSON));
    UdonModuleInfo::set_default_module_info_path(module_info_path.clone())
        .map_err(|e| anyhow::anyhow!("failed to configure module info path: {}", e))?;
    info!(level = %cli.log_level, "logging initialized");
    info!(module_info = %module_info_path.display(), "module info path configured");

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
    info!(
        mode = ?mode,
        input = %input.display(),
        output = ?output.map(|x| x.display().to_string()),
        template = ?template.map(|x| x.display().to_string()),
        "start command"
    );
    if !input.exists() {
        bail!("input path does not exist: {}", input.display());
    }

    if mode != Mode::Asm && template.is_some() {
        bail!("--template is only valid for asm command");
    }

    if input.is_file() {
        cli_process_single_file(mode, input, output, template)?;
        return Ok(());
    }

    if !input.is_dir() {
        bail!(
            "input path is neither file nor directory: {}",
            input.display()
        );
    }

    cli_process_directory(mode, input, output, template)
}

fn cli_process_single_file(
    mode: Mode,
    input_file: &Path,
    output: Option<&Path>,
    template: Option<&Path>,
) -> Result<()> {
    debug!(
        mode = ?mode,
        input = %input_file.display(),
        output = ?output.map(|x| x.display().to_string()),
        template = ?template.map(|x| x.display().to_string()),
        "processing single file"
    );
    ensure_input_extension(mode, input_file)?;
    validate_template_kind_for_single(mode, template)?;
    let output_file = process_single_file(mode, input_file, output, None, template)?;
    info!("{} -> {}", input_file.display(), output_file.display());
    Ok(())
}

fn cli_process_directory(
    mode: Mode,
    input_dir: &Path,
    output: Option<&Path>,
    template: Option<&Path>,
) -> Result<()> {
    info!(
        mode = ?mode,
        input_dir = %input_dir.display(),
        output = ?output.map(|x| x.display().to_string()),
        template = ?template.map(|x| x.display().to_string()),
        "processing directory"
    );
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
    debug!(count = input_files.len(), "matched input files");

    // todo: parallel
    for input_file in input_files {
        let file_template = match mode {
            Mode::Asm => template,
            _ => None,
        };
        let output_file =
            process_single_file(mode, &input_file, None, Some(&output_dir), file_template)?;
        info!("{} -> {}", input_file.display(), output_file.display());
    }

    Ok(())
}

fn process_single_file(
    mode: Mode,
    input_file: &Path,
    output: Option<&Path>,
    output_dir: Option<&Path>,
    template: Option<&Path>,
) -> Result<PathBuf> {
    let mut prepared = prepare_single_input(mode, input_file)?;
    let default_filename = default_output_filename(mode, input_file, &prepared);
    let output_file = if let Some(output_dir) = output_dir {
        output_dir.join(default_filename)
    } else {
        match output {
            None => input_file
                .parent()
                .unwrap_or(Path::new("."))
                .join(default_filename),
            Some(path) if path.is_dir() => path.join(default_filename),
            Some(path) => path.to_path_buf(),
        }
    };

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }
    process_single_file_inner(mode, input_file, &output_file, template, &mut prepared)?;
    Ok(output_file)
}

fn process_single_file_inner(
    mode: Mode,
    input_file: &Path,
    output_file: &Path,
    template: Option<&Path>,
    prepared: &mut PreparedSingleInput,
) -> Result<()> {
    debug!(
        mode = ?mode,
        input = %input_file.display(),
        output = %output_file.display(),
        template = ?template.map(|x| x.display().to_string()),
        "processing item"
    );
    match mode {
        Mode::Dc => {
            let PreparedSingleInput::Dc { ctx } = prepared else {
                bail!("internal error: expected prepared dc input");
            };
            let output = ctx.run_decompile()?;
            fs::write(output_file, output.generated_code)
                .with_context(|| format!("failed to write {}", output_file.display()))?;
            debug!(
                variables = ctx.variables.variables.len(),
                basic_blocks = ctx.basic_blocks.blocks.len(),
                cfg_functions = ctx.cfg_functions.len(),
                "wrote dc output from basic decompile pipeline"
            );
        }
        Mode::Dasm => {
            let PreparedSingleInput::Dasm {
                program,
                bind_analysis,
                instruction_comments,
                ..
            } = prepared
            else {
                bail!("internal error: expected prepared dasm input");
            };
            let asm = disassemble_program_to_text(
                program,
                &bind_analysis.binds,
                &bind_analysis.bind_tables,
                instruction_comments,
            )
            .with_context(|| {
                format!(
                    "failed to disassemble program from {}",
                    input_file.display()
                )
            })?;
            let source_name = input_file
                .file_name()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or_else(|| input_file.display().to_string());
            let asm_with_source = format!("; source-program: {}\n{}", source_name, asm);
            fs::write(output_file, asm_with_source)
                .with_context(|| format!("failed to write {}", output_file.display()))?;
            debug!("wrote disassembly output");
        }
        Mode::Asm => {
            let PreparedSingleInput::Asm { asm_text } = prepared else {
                bail!("internal error: expected prepared asm input");
            };
            let template_path =
                choose_hex_template_path(input_file, output_file, template, asm_text)?;
            let assembled_hex = if is_asset_file(&template_path) {
                let original_compressed = read_compressed_program_bytes_from_asset(&template_path)?;
                let original_hex = original_compressed
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                assemble_hex_with_original(&original_hex, asm_text).with_context(|| {
                    format!(
                        "failed to assemble {} using asset template {}",
                        input_file.display(),
                        template_path.display()
                    )
                })?
            } else {
                let original_hex = std::fs::read_to_string(&template_path)
                    .with_context(|| format!("failed to read {}", template_path.display()))?;
                assemble_hex_with_original(&original_hex, asm_text).with_context(|| {
                    format!(
                        "failed to assemble {} using template {}",
                        input_file.display(),
                        template_path.display()
                    )
                })?
            };
            fs::write(output_file, assembled_hex)
                .with_context(|| format!("failed to write {}", output_file.display()))?;
            debug!(template = %template_path.display(), "wrote assembled hex output");
        }
    }
    Ok(())
}

fn prepare_single_input(mode: Mode, input_file: &Path) -> Result<PreparedSingleInput> {
    match mode {
        Mode::Dc => {
            let ctx = DecompileContext::from_file(input_file).with_context(|| {
                format!(
                    "failed to load decompile context from {}",
                    input_file.display()
                )
            })?;
            Ok(PreparedSingleInput::Dc { ctx: Box::new(ctx) })
        }
        Mode::Dasm => {
            let bytes = read_program_bytes(input_file)?;
            let program = UdonProgramBinary::parse_bytes(&bytes).with_context(|| {
                format!("failed to parse program from {}", input_file.display())
            })?;
            let mut ctx = DecompileContext::from_program(&program).with_context(|| {
                format!(
                    "failed to create decompile context from {}",
                    input_file.display()
                )
            })?;
            ctx.set_input_file_name(
                input_file
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string()),
            );
            ctx.run_analysis().with_context(|| {
                format!(
                    "failed to analyze program for bind reconstruction from {}",
                    input_file.display()
                )
            })?;
            let bind_analysis = collect_asm_bind_analysis(&ctx).with_context(|| {
                format!(
                    "failed to collect disassembly binds from {}",
                    input_file.display()
                )
            })?;
            let instruction_comments =
                collect_asm_instruction_comments(&ctx).with_context(|| {
                    format!(
                        "failed to collect disassembly comments from {}",
                        input_file.display()
                    )
                })?;
            let output_stem = ctx.infer_output_stem_for_file();
            Ok(PreparedSingleInput::Dasm {
                program,
                bind_analysis,
                instruction_comments,
                output_stem,
            })
        }
        Mode::Asm => {
            let asm_text = fs::read_to_string(input_file)
                .with_context(|| format!("failed to read {}", input_file.display()))?;
            Ok(PreparedSingleInput::Asm { asm_text })
        }
    }
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

fn choose_hex_template_path(
    input_asm: &Path,
    output_hex: &Path,
    explicit_template: Option<&Path>,
    asm_text: &str,
) -> Result<PathBuf> {
    if let Some(template) = explicit_template {
        if template.exists() && template.is_file() {
            return Ok(template.to_path_buf());
        }
        if template.exists() && template.is_dir() {
            if let Some(hint_name) = extract_source_program_hint(asm_text) {
                let hinted = template.join(hint_name);
                if hinted.exists() && hinted.is_file() {
                    return Ok(hinted);
                }
            }
            let by_stem = template.join(format!("{}.{}", input_file_stem(input_asm), EXT_HEX));
            if by_stem.exists() && by_stem.is_file() {
                return Ok(by_stem);
            }
            let by_asset_stem =
                template.join(format!("{}.{}", input_file_stem(input_asm), EXT_ASSET));
            if by_asset_stem.exists() && by_asset_stem.is_file() {
                return Ok(by_asset_stem);
            }
            bail!(
                "template directory does not contain matching hex/asset for {} (tried hint and stem)",
                input_asm.display()
            );
        }
        bail!("template path does not exist: {}", template.display());
    }

    if output_hex.exists() && output_hex.is_file() {
        return Ok(output_hex.to_path_buf());
    }
    if let Some(hint_name) = extract_source_program_hint(asm_text) {
        let hinted = input_asm.parent().unwrap_or(Path::new(".")).join(hint_name);
        if hinted.exists() && hinted.is_file() {
            return Ok(hinted);
        }
    }
    let sibling = input_asm.with_extension(EXT_HEX);
    if sibling.exists() && sibling.is_file() {
        return Ok(sibling);
    }
    let sibling_asset = input_asm.with_extension(EXT_ASSET);
    if sibling_asset.exists() && sibling_asset.is_file() {
        return Ok(sibling_asset);
    }
    bail!(
        "asm requires a hex or asset template file. provide --template, or ensure existing output/sibling hex/asset is present"
    );
}

fn extract_source_program_hint(asm_text: &str) -> Option<String> {
    let first_line = asm_text.lines().next()?.trim();
    for prefix in ["; source-program:", "; source-hex:"] {
        if let Some(stripped) = first_line.strip_prefix(prefix) {
            let value = stripped.trim();
            if value.is_empty() {
                return None;
            }
            return Some(value.to_string());
        }
    }
    None
}

fn ensure_input_extension(mode: Mode, input_file: &Path) -> Result<()> {
    let ext = input_file
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let ok = match mode {
        Mode::Dc | Mode::Dasm => ext == EXT_HEX || ext == EXT_ASSET,
        Mode::Asm => ext == EXT_ASM,
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
            Mode::Dc | Mode::Dasm => ext == EXT_HEX || ext == EXT_ASSET,
            Mode::Asm => ext == EXT_ASM,
        };
        if matched {
            out.push(path);
        }
    }
    Ok(out)
}

fn default_output_filename(
    mode: Mode,
    input_file: &Path,
    prepared: &PreparedSingleInput,
) -> String {
    match mode {
        Mode::Dc => {
            let PreparedSingleInput::Dc { ctx } = prepared else {
                panic!()
            };
            format!("{}.{}", ctx.infer_output_stem_for_file(), EXT_CS)
        }
        Mode::Dasm => {
            let PreparedSingleInput::Dasm { output_stem, .. } = prepared else {
                panic!()
            };
            format!("{output_stem}.{}", EXT_ASM)
        }
        Mode::Asm => format!("{}.{}", input_file_stem(input_file), EXT_HEX),
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
        Mode::Dc | Mode::Dasm => "*.{hex,asset}",
        Mode::Asm => INPUT_GLOB_ASM,
    }
}

fn is_asset_file(path: &Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x == EXT_ASSET)
}
