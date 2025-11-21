use proptest::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// Helper to ensure parent directories exist
fn ensure_parent_exists(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
}

// Run node repomix and extract file paths
fn run_node_repomix_files(input_dir: &Path) -> HashSet<String> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let bin_path = repo_root.join("bin/repomix.cjs");
    let output_path = input_dir.join("node_output.xml");

    // Using --style plain might be easier to parse, or just XML and regex/grep
    // Let's use XML and simple parsing as before, it's reliable enough for file lists
    let output = Command::new("node")
        .arg(bin_path)
        .arg(input_dir)
        .arg("--style")
        .arg("xml")
        .arg("--output")
        .arg(&output_path)
        .arg("--no-file-summary") // minimize output
        .arg("--no-directory-structure") // minimize output
        .output()
        .expect("Failed to run node repomix");

    if !output.status.success() {
        // It might fail if no files are found?
        // Repomix usually output "No files found" or similar but exit code might be 0 or 1
        // Let's check stdout/stderr if needed, but for now assume success
        // println!("Node stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    if !output_path.exists() {
        return HashSet::new();
    }

    let content = fs::read_to_string(&output_path).expect("Failed to read node output");
    extract_file_paths(&content)
}

// Run rust repomix and extract file paths
fn run_rust_repomix_files(input_dir: &Path) -> HashSet<String> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    
    // Try release binary first, then debug
    let bin_path = if repo_root.join("target/release/repomix-rs").exists() {
        repo_root.join("target/release/repomix-rs")
    } else {
        repo_root.join("target/debug/repomix-rs")
    };

    let output_path = input_dir.join("rust_output.xml");
    let config_path = input_dir.join("repomix.config.json");
    
    let config_content = r#"{
        "output": {
            "fileSummary": false,
            "directoryStructure": false
        }
    }"#;
    fs::write(&config_path, config_content).expect("Failed to write rust config");

    let output = Command::new(&bin_path)
        .arg(input_dir)
        .arg("--config")
        .arg(&config_path)
        .arg("--style")
        .arg("xml")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to run rust repomix");

    if !output.status.success() {
        println!("Rust Repomix Failed!");
        println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    if !output_path.exists() {
        println!("Rust output file not created at {:?}", output_path);
        return HashSet::new();
    }

    let content = fs::read_to_string(&output_path).expect("Failed to read rust output");
    let paths = extract_file_paths(&content);
    
    if paths.is_empty() {
        println!("Rust found NO files in {:?}", input_dir);
        // println!("Rust stdout: {}", String::from_utf8_lossy(&output.stdout));
        // println!("Rust stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    paths
}
fn extract_file_paths(content: &str) -> HashSet<String> {
    let mut paths = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<file path=\"") {
            if let Some(start) = trimmed.find("path=\"") {
                let start = start + 6;
                if let Some(end) = trimmed[start..].find("\"") {
                    let path = &trimmed[start..start + end];
                    paths.insert(path.to_string().replace('\\', "/"));
                }
            }
        }
    }
    paths
}

proptest! {
    // Generate more complex file structures:
    // - Empty files
    // - Dot files (hidden)
    // - Files in nested directories
    // - Files with specific extensions that might be ignored (e.g., .log, .git)
    // - Binary-looking files (though we just check path existence, content checks are separate)
    #![proptest_config(ProptestConfig::with_cases(20))] // Run 20 cases
    #[test]
    fn test_file_collection_parity(
        files in prop::collection::hash_map(
            // Path pattern: nested dirs, dot files, common extensions
            "[a-zA-Z0-9_.-]{1,10}(/[a-zA-Z0-9_.-]{1,10}){0,3}",
            // Content: empty or some text
            "[a-zA-Z0-9 ]{0,100}",
            0..20 // Number of files
        )
    ) {
        let dir = TempDir::new().unwrap();
        let input_dir = dir.path().join("input");
        fs::create_dir(&input_dir).unwrap();
        
        // Setup files
        for (path, content) in &files {
            // normalize path to prevent creation errors (e.g. ending with / or empty parts)
            if path.ends_with('/') || path.contains("//") { continue; }
            
            let full_path = input_dir.join(path);
            ensure_parent_exists(&full_path);
            if let Err(_) = fs::write(&full_path, content) {
                continue; // skip invalid paths generated
            }
        }

        // Special case: Ensure some empty files and dotfiles explicitly if random didn't generate them enough
        // (Proptest random might cover it, but let's be sure)
        let extras = vec![
            ".hidden",
            "visible.txt",
            "empty_file.txt", // 0 bytes
            "subdir/.hidden_in_subdir",
            "subdir/empty_in_subdir",
        ];
        for extra in extras {
            let p = input_dir.join(extra);
            ensure_parent_exists(&p);
            // Write empty content for "empty" files
            if extra.contains("empty") {
                fs::write(&p, "").unwrap();
            } else {
                fs::write(&p, "content").unwrap();
            }
        }
        
        // Create a dummy .gitignore to test ignore logic too?
        // For now let's test default behavior without custom .gitignore
        
        let node_files = run_node_repomix_files(&input_dir);
        let rust_files = run_rust_repomix_files(&input_dir);
        
        // Filter out the output files themselves if they ended up in the set (though run functions put them in input_dir)
        // Wait, run functions put output files INSIDE input_dir? 
        // "let output_path = input_dir.join(\"node_output.xml\");" -> Yes.
        // Repomix by default ignores its own output, but let's be safe and filter.
        let filter_outputs = |set: HashSet<String>| -> HashSet<String> {
            set.into_iter()
                .filter(|p| !p.ends_with("node_output.xml") && !p.ends_with("rust_output.xml") && !p.ends_with("repomix.config.json"))
                .collect()
        };
        
        let node_files = filter_outputs(node_files);
        let rust_files = filter_outputs(rust_files);

        // Compare
        let node_only: Vec<_> = node_files.difference(&rust_files).collect();
        let rust_only: Vec<_> = rust_files.difference(&node_files).collect();
        
        if !node_only.is_empty() || !rust_only.is_empty() {
             // List all files in input dir for debugging context
            let mut all_files_on_disk = Vec::new();
            for entry in walkdir::WalkDir::new(&input_dir) {
                let entry = entry.unwrap();
                if entry.file_type().is_file() {
                    let path = entry.path().strip_prefix(&input_dir).unwrap();
                    all_files_on_disk.push(path.to_string_lossy().to_string());
                }
            }
            all_files_on_disk.sort();
            
            panic!(
                "File collection mismatch!\n\x20Node only ({:?}): {:?}\n\x20Rust only ({:?}): {:?}\n\x20All files on disk: {:#?}",
                node_only.len(), node_only,
                rust_only.len(), rust_only,
                all_files_on_disk
            );
        }
    }
}
