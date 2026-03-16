use std::fs;
use std::path::{Path, PathBuf};

use udon_decompiler::decompiler::DecompileContext;

fn cases_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cases")
}

fn collect_case_paths(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::<PathBuf>::new();
    collect_md_recursive(root, &mut out);
    out.sort();
    out
}

fn collect_md_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_recursive(&path, out);
            continue;
        }

        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        if path.file_name().and_then(|x| x.to_str()) == Some("README.md") {
            continue;
        }
        out.push(path);
    }
}

fn parse_markdown_code_fences(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::<(String, String)>::new();
    let parts = text.split("```").collect::<Vec<_>>();

    for idx in (1..parts.len()).step_by(2) {
        let raw = parts[idx];
        let mut lines = raw.lines();
        let lang = lines.next().unwrap_or_default().trim().to_string();
        let content = lines.collect::<Vec<_>>().join("\n");
        out.push((lang, format!("{content}\n")));
    }

    out
}

fn load_hex_from_case(case_path: &Path) -> Option<String> {
    let text = fs::read_to_string(case_path).ok()?;
    let blocks = parse_markdown_code_fences(&text);
    for (lang, content) in blocks {
        if lang.eq_ignore_ascii_case("hex") {
            return Some(content);
        }
    }
    None
}

fn snapshot_name_for_case(case_path: &Path, root: &Path) -> String {
    let rel = case_path.strip_prefix(root).unwrap_or(case_path);
    let mut parts = rel
        .iter()
        .map(|x| x.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if let Some(last) = parts.last_mut()
        && let Some(stripped) = last.strip_suffix(".md")
    {
        *last = stripped.to_string();
    }

    parts
        .into_iter()
        .map(|part| {
            part.chars()
                .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("__")
}

#[test]
fn e2e_smoke() {
    let root = cases_root();
    assert!(root.exists(), "e2e root missing: {}", root.display());

    let cases = collect_case_paths(&root);
    assert!(!cases.is_empty(), "no markdown cases found");

    let mut failures = Vec::<String>::new();
    for case_path in cases {
        let Some(hex_text) = load_hex_from_case(&case_path) else {
            continue;
        };

        let file_name = case_path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("case.md")
            .to_string();

        let result = (|| {
            let mut ctx = DecompileContext::from_compressed_hex_text(&hex_text, Some(file_name))?;
            let _ = ctx.run_decompile()?;
            Ok::<(), udon_decompiler::decompiler::DecompileError>(())
        })();

        if let Err(e) = result {
            failures.push(format!("{}: {}", case_path.display(), e));
        }
    }

    assert!(
        failures.is_empty(),
        "e2e smoke failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn e2e_snapshot() {
    let root = cases_root();
    assert!(root.exists(), "e2e root missing: {}", root.display());

    let cases = collect_case_paths(&root);
    assert!(!cases.is_empty(), "no markdown cases found");

    for case_path in cases {
        let Some(hex_text) = load_hex_from_case(&case_path) else {
            continue;
        };

        let stem = case_path
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or("case")
            .to_string();

        let mut ctx = DecompileContext::from_compressed_hex_text(
            &hex_text,
            Some(format!("{stem}.hex")),
        )
        .expect("load context from compressed hex");
        let output = ctx.run_decompile().expect("run pipeline");

        let snapshot_name = snapshot_name_for_case(&case_path, &root);
        insta::assert_snapshot!(snapshot_name, output.generated_code);
    }
}
