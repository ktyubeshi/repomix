use proptest::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn ensure_parent_exists(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
}

fn run_node_repomix(input_dir: &Path, output_file: &Path) -> String {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let bin_path = repo_root.join("bin/repomix.cjs");

    // Use absolute paths for safety
    let input_abs = input_dir.canonicalize().unwrap();
    // Output file might not exist, so we can't canonicalize it directly if it doesn't exist.
    // But we can canonicalize the parent.
    let output_abs = if output_file.is_absolute() {
        output_file.to_path_buf()
    } else {
        // If it's relative, join with current dir (which shouldn't happen in this test logic but safe to handle)
        std::env::current_dir().unwrap().join(output_file)
    };

    // Ensure output dir exists
    if let Some(parent) = output_abs.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let output = Command::new("node")
        .arg(bin_path)
        .arg(&input_abs)
        .arg("--style")
        .arg("xml")
        .arg("--output")
        .arg(&output_abs)
        // Add ignore to avoid including output files in each other if they are close (though here they are separate)
        .output()
        .expect("Failed to run node repomix");

    if !output.status.success() {
        panic!(
            "Node repomix failed: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fs::read_to_string(&output_abs).expect("Failed to read node output")
}

fn run_rust_repomix(input_dir: &Path, output_file: &Path) -> String {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bin_path = repo_root.join("target/debug/repomix-rs");

    let input_abs = input_dir.canonicalize().unwrap();
    let output_abs = if output_file.is_absolute() {
        output_file.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(output_file)
    };

    if let Some(parent) = output_abs.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let output = Command::new(bin_path)
        .arg(&input_abs)
        .arg("--style")
        .arg("xml")
        .arg("--output")
        .arg(&output_abs)
        .output()
        .expect("Failed to run rust repomix");

    if !output.status.success() {
        panic!(
            "Rust repomix failed: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fs::read_to_string(&output_abs).expect("Failed to read rust output")
}

fn parse_output(content: &str, root: &Path) -> HashMap<String, String> {
    let mut files = HashMap::new();
    let mut current_path = None;
    let mut current_content = String::new();
    let mut in_file = false;

    let root_abs = root.canonicalize().unwrap_or(root.to_path_buf());

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<file path=\"") {
            let start = trimmed.find("path=\"").unwrap() + 6;
            let end = trimmed[start..].find("\"").unwrap() + start;
            let path_str = &trimmed[start..end];

            let path = Path::new(path_str);

            // Normalize path
            let relative_path = if path.is_absolute() {
                match path.strip_prefix(&root_abs) {
                    Ok(p) => p.to_string_lossy().into_owned(),
                    Err(_) => path_str.to_string(),
                }
            } else {
                path_str.to_string()
            };

            let relative_path = relative_path.replace('\\', "/");

            current_path = Some(relative_path);
            current_content.clear();
            in_file = true;
        } else if trimmed == "</file>" {
            if let Some(path) = current_path.take() {
                files.insert(path, current_content.trim().to_string());
            }
            in_file = false;
        } else if in_file {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    files
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]
    #[test]
    fn test_repomix_comparison(
        files in prop::collection::hash_map("[a-z0-9]{1,5}(/[a-z0-9]{1,5})*\\.txt", "[a-z0-9 \\n]{0,50}", 1..5)
    ) {
        let dir = TempDir::new().unwrap();
        let input_dir = dir.path().join("input");
        fs::create_dir(&input_dir).unwrap();

        for (name, content) in &files {
            let file_path = input_dir.join(name);
            ensure_parent_exists(&file_path);
            fs::write(&file_path, content).unwrap();
        }

        let node_out_path = dir.path().join("node_out.xml");
        let rust_out_path = dir.path().join("rust_out.xml");

        let node_out = run_node_repomix(&input_dir, &node_out_path);
        let rust_out = run_rust_repomix(&input_dir, &rust_out_path);

        // Verify header presence in both
        assert!(node_out.contains("This file is a merged representation"), "Node output missing header");
        assert!(rust_out.contains("This file is a merged representation"), "Rust output missing header");
        assert!(rust_out.contains("<file_summary>"), "Rust output missing file_summary");

        // Verify parsed content
        let node_files = parse_output(&node_out, &input_dir);
        let rust_files = parse_output(&rust_out, &input_dir);

        let mut node_keys: Vec<_> = node_files.keys().collect();
        let mut rust_keys: Vec<_> = rust_files.keys().collect();
        node_keys.sort();
        rust_keys.sort();

        assert_eq!(node_keys, rust_keys, "File list mismatch.\nNode: {:?}\nRust: {:?}", node_keys, rust_keys);

        for key in node_keys {
            let node_content = node_files.get(key).unwrap();
            let rust_content = rust_files.get(key).unwrap();
            // trim because one might have extra newline from template or parsing
            assert_eq!(node_content.trim(), rust_content.trim(), "Content mismatch for {}", key);
        }
    }
}
