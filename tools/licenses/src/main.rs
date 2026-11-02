//! Collects the license texts of every library compiled into NeoCross into one file that ships
//! with each release: `THIRD_PARTY_LICENSES.txt`.
//!
//! `cargo run --release --manifest-path tools/licenses/Cargo.toml -- --target <triple> --out <file>`

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(target), Some(out)) = (arg(&args, "--target"), arg(&args, "--out")) else {
        eprintln!("usage: neocross-licenses --target <triple> --out <file> [--manifest Cargo.toml]");
        return ExitCode::FAILURE;
    };
    let manifest = arg(&args, "--manifest").unwrap_or_else(|| "Cargo.toml".into());
    match run(&manifest, &target, Path::new(&out)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(manifest: &str, target: &str, out_path: &Path) -> Result<(), String> {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked", "--filter-platform", target, "--manifest-path", manifest])
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let meta: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    // Exactly what ends up in the executable for this target: normal dependencies with the
    // features really enabled (cargo metadata would also list unused optional ones).
    let tree = std::process::Command::new("cargo")
        .args(["tree", "--locked", "-e", "normal", "--target", target, "--prefix", "none", "--format", "{p}"])
        .args(["--manifest-path", manifest])
        .output()
        .map_err(|e| format!("cargo tree: {e}"))?;
    if !tree.status.success() {
        return Err(String::from_utf8_lossy(&tree.stderr).into_owned());
    }
    let root = meta["resolve"]["root"].as_str().ok_or("no root package")?.to_string();
    let used: BTreeSet<String> = String::from_utf8_lossy(&tree.stdout)
        .lines()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            Some(format!("{}@{}", parts.next()?, parts.next()?.trim_start_matches('v')))
        })
        .collect();

    let mut entries: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    for pkg in meta["packages"].as_array().ok_or("no packages")? {
        let name = pkg["name"].as_str().unwrap_or("?");
        let version = pkg["version"].as_str().unwrap_or("?");
        if pkg["id"].as_str() == Some(root.as_str()) || !used.contains(&format!("{name}@{version}")) {
            continue;
        }
        let license = pkg["license"].as_str().unwrap_or("see license text");
        let repo = pkg["repository"].as_str().unwrap_or("");
        let dir = Path::new(pkg["manifest_path"].as_str().unwrap_or_default()).parent().map(Path::to_path_buf);
        let mut texts = Vec::new();
        if let Some(Ok(rd)) = dir.map(std::fs::read_dir) {
            let mut files: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .filter(|p| {
                    let n = p.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_ascii_uppercase();
                    ["LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT", "UNLICENSE"].iter().any(|k| n.starts_with(k))
                })
                .collect();
            files.sort();
            texts.extend(files.iter().filter_map(|f| std::fs::read_to_string(f).ok()).map(|t| t.replace("\r\n", "\n").trim().to_string()));
        }
        let line = format!("{name} {version} — {license}{}", if repo.is_empty() { String::new() } else { format!(" — {repo}") });
        entries.insert(format!("{name}@{version}"), (line, texts));
    }

    let mut by_text: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut without = Vec::new();
    for (key, (_, texts)) in &entries {
        if texts.is_empty() {
            without.push(key.as_str());
        }
        for t in texts {
            by_text.entry(t).or_default().push(key);
        }
    }

    let mut out = String::new();
    out.push_str("NeoCross – licenses of the open-source libraries it uses\n");
    out.push_str("NeoCross – Lizenzen der verwendeten Open-Source-Bibliotheken\n");
    out.push_str("============================================================\n\n");
    out.push_str(&format!("Target: {target}\n"));
    out.push_str("NeoCross © 2026 T-GOW. The following libraries are distributed under their own licenses.\n\n");
    for (line, _) in entries.values() {
        out.push_str(line);
        out.push('\n');
    }
    if !without.is_empty() {
        out.push_str("\nNo license file shipped in the package (license as declared above):\n");
        out.push_str(&without.join(", "));
        out.push('\n');
    }
    for (text, users) in &by_text {
        out.push_str("\n\n------------------------------------------------------------\n");
        out.push_str(&users.join(", "));
        out.push_str("\n------------------------------------------------------------\n\n");
        out.push_str(text);
        out.push('\n');
    }
    if let Some(dir) = out_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(out_path, out).map_err(|e| e.to_string())?;
    println!("{} libraries, {} distinct license texts -> {}", entries.len(), by_text.len(), out_path.display());
    Ok(())
}
