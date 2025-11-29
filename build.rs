use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use bento::{Compiler, OptimizationLevel, Request, ShaderLang};
use dashi::ShaderType;

/// Recursively traverses a directory and collects all *.slang files.
fn collect_slang_files(dir: &Path, slang_files: &mut Vec<PathBuf>) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to get entry");
            let path = entry.path();
            if path.is_dir() {
                collect_slang_files(&path, slang_files);
            } else if let Some(ext) = path.extension() {
                if ext == "slang" {
                    slang_files.push(path);
                }
            }
        }
    }
}

fn infer_stage(path: &Path) -> Option<ShaderType> {
    let name = path.file_name()?.to_string_lossy();

    if name.contains("vert") {
        Some(ShaderType::Vertex)
    } else if name.contains("frag") {
        Some(ShaderType::Fragment)
    } else if name.contains("comp") {
        Some(ShaderType::Compute)
    } else {
        None
    }
}

fn expand_imports(
    path: &Path,
    include_dir: &Path,
    visiting: &mut HashSet<PathBuf>,
    included: &mut HashSet<PathBuf>,
) -> Result<String, Box<dyn std::error::Error>> {
    if !visiting.insert(path.to_path_buf()) {
        return Err(format!("Cyclic import detected for {}", path.display()).into());
    }

    let source = fs::read_to_string(path)?;
    let mut output = String::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") && trimmed.ends_with(';') {
            let import_name = trimmed
                .trim_start_matches("import")
                .trim()
                .trim_end_matches(';')
                .trim();

            let import_path = include_dir.join(format!("{import_name}.slang"));
            if included.insert(import_path.clone()) {
                let imported = expand_imports(&import_path, include_dir, visiting, included)?;
                output.push_str(&imported);
                output.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    visiting.remove(path);
    Ok(output)
}

fn main() {
    // Get the source and output directories from environment variables.
    let out_dir = env::var("SLANG_OUT_DIR").unwrap_or_else(|_| "target/spirv".to_string());
    let src_dir = env::var("SLANG_SRC_DIR").unwrap_or_else(|_| "src/slang/src".to_string());
    let include_dir = env::var("SLANG_INCLUDE_DIR").unwrap_or_else(|_| "src/slang/include".to_string());

    let src_path = Path::new(&src_dir);
    let out_path = Path::new(&out_dir);
    let include_path = Path::new(&include_dir);

    // Ensure the output directory exists.
    if !out_path.exists() {
        fs::create_dir_all(out_path).expect("Failed to create output directory");
    }

    // Track changes to include files as well as source files.
    let mut include_files = Vec::new();
    collect_slang_files(include_path, &mut include_files);
    for include_file in &include_files {
        println!(
            "cargo:rerun-if-changed={}",
            include_file.to_str().expect("Invalid include file path")
        );
    }

    // Collect all .slang files from the source directory.
    let mut slang_files = Vec::new();
    collect_slang_files(src_path, &mut slang_files);

    let compiler = Compiler::new().expect("Failed to create Bento compiler");

    // Compile each .slang file with Bento.
    for input_file in &slang_files {
        println!(
            "cargo:rerun-if-changed={}",
            input_file.to_str().expect("Invalid input file path")
        );

        let stage = infer_stage(input_file)
            .unwrap_or_else(|| panic!("Unable to infer shader stage for {}", input_file.display()));

        let relative_path = input_file.strip_prefix(src_path).expect("Failed to strip prefix");
        let mut output_file = out_path.join(relative_path);
        output_file.set_extension("bto");

        // Ensure the output subdirectory exists.
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).expect("Failed to create output subdirectories");
        }

        let mut visiting = HashSet::new();
        let mut included = HashSet::new();
        let expanded_source = expand_imports(input_file, include_path, &mut visiting, &mut included)
            .unwrap_or_else(|err| panic!("Failed to expand imports for {}: {err}", input_file.display()));

        let request = Request {
            name: Some(
                input_file
                    .file_stem()
                    .expect("Missing file stem")
                    .to_string_lossy()
                    .to_string(),
            ),
            lang: ShaderLang::Slang,
            stage,
            optimization: OptimizationLevel::Performance,
            debug_symbols: true,
        };

        let result = compiler
            .compile(expanded_source.as_bytes(), &request)
            .unwrap_or_else(|err| panic!("Failed to compile {}: {err}", input_file.display()));

        result
            .save_to_disk(output_file.to_str().expect("Invalid output path"))
            .unwrap_or_else(|err| panic!("Failed saving {}: {err}", output_file.display()));

        println!(
            "Compiled {} to {}",
            input_file.display(),
            output_file.display()
        );
    }
}
