use crate::config::schema::RepomixConfig;
use crate::core::compress;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

const DATA_URI_MIN_LEN: usize = 40;
const STANDALONE_MIN_LEN: usize = 60;
const TRUNCATION_LEN: usize = 32;

pub fn process_content(
    content: &str,
    path: &Path,
    config: &RepomixConfig,
) -> anyhow::Result<String> {
    let mut processed = content.to_string();

    if config.output.truncate_base64 {
        processed = truncate_base64_content(&processed);
    }

    if config.output.remove_comments {
        processed = strip_comments(&processed, path);
    }

    if config.output.remove_empty_lines {
        processed = remove_empty_lines(&processed);
    }

    processed = processed.trim().to_string();

    if config.output.compress {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        processed = compress::compress_content(&processed, ext).unwrap_or(processed);
    }

    Ok(processed)
}

fn strip_comments(content: &str, path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        // Languages that commonly use // and /* */ comments
        "rs" | "ts" | "tsx" | "js" | "jsx" | "go" | "c" | "cc" | "cpp" | "cxx" | "h" | "hpp"
        | "hh" | "java" | "kt" | "kts" | "swift" | "cs" | "scala" | "php" => {
            strip_c_like_comments(content)
        }
        // Hash comment languages
        "py" | "rb" | "sh" | "bash" | "yaml" | "yml" | "toml" | "hs" => {
            strip_hash_comments(content)
        }
        _ => content.to_string(),
    }
}

fn strip_c_like_comments(content: &str) -> String {
    static BLOCK: Lazy<Regex> = Lazy::new(|| Regex::new("(?s)/\\*.*?\\*/").unwrap());
    static LINE: Lazy<Regex> = Lazy::new(|| Regex::new("(?m)//.*?$").unwrap());

    let without_block = BLOCK.replace_all(content, |caps: &regex::Captures| {
        let text = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        let newlines = text.matches('\n').count();
        "\n".repeat(newlines)
    });

    LINE.replace_all(&without_block, "").to_string()
}

fn strip_hash_comments(content: &str) -> String {
    static HASH: Lazy<Regex> = Lazy::new(|| Regex::new("(?m)^\\s*#.*$").unwrap());
    HASH.replace_all(content, "").to_string()
}

fn remove_empty_lines(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
}

fn truncate_base64_content(content: &str) -> String {
    static DATA_URI_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(&format!(
            "data:([A-Za-z0-9/\\-\\+]+)(;[A-Za-z0-9\\-=]+)*;base64,([A-Za-z0-9+/=]{{{},}})",
            DATA_URI_MIN_LEN
        ))
        .unwrap()
    });
    static STANDALONE_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(&format!(
            "([A-Za-z0-9+/]{{{},}}={{0,2}})",
            STANDALONE_MIN_LEN
        ))
        .unwrap()
    });

    let mut result = DATA_URI_RE
        .replace_all(content, |caps: &regex::Captures| {
            let mime = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let params = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let data = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let preview = &data[..std::cmp::min(TRUNCATION_LEN, data.len())];
            format!("data:{mime}{params};base64,{preview}...")
        })
        .to_string();

    result = STANDALONE_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let data = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if is_likely_base64(data) {
                let preview = &data[..std::cmp::min(TRUNCATION_LEN, data.len())];
                format!("{preview}...")
            } else {
                data.to_string()
            }
        })
        .to_string();

    result
}

fn is_likely_base64(data: &str) -> bool {
    if !Regex::new("^[A-Za-z0-9+/]+=*$").unwrap().is_match(data) {
        return false;
    }

    let unique_chars = data.chars().collect::<HashSet<char>>();
    if unique_chars.len() < 10 {
        return false;
    }

    let has_numbers = data.chars().any(|c| c.is_ascii_digit());
    let has_upper = data.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = data.chars().any(|c| c.is_ascii_lowercase());
    let has_special = data.chars().any(|c| c == '+' || c == '/');

    let categories = [has_numbers, has_upper, has_lower, has_special]
        .iter()
        .filter(|v| **v)
        .count();

    categories >= 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn truncates_data_uri() {
        let path = PathBuf::from("image.js");
        let base64 =
            "QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVo=QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVo=";
        let original = format!("const img = \"data:image/png;base64,{base64}\";");
        let processed = process_content(&original, &path, &config_with(true, false, false)).unwrap();
        assert!(processed.contains("data:image/png;base64,"));
        assert!(processed.contains("..."));
        assert!(!processed.contains("QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVo="));
    }

    #[test]
    fn strips_c_like_comments() {
        let path = PathBuf::from("main.rs");
        let content = "fn main() { // comment\n  /* block */ println!(\"hi\");\n}";
        let processed = process_content(content, &path, &config_with(false, true, false)).unwrap();
        assert!(!processed.contains("// comment"));
        assert!(!processed.contains("/* block */"));
    }

    #[test]
    fn removes_empty_lines() {
        let path = PathBuf::from("script.py");
        let content = "line1\n\n\nline2\n";
        let processed = process_content(content, &path, &config_with(false, false, true)).unwrap();
        assert_eq!(processed, "line1\nline2");
    }

    fn config_with(
        truncate_base64: bool,
        remove_comments: bool,
        remove_empty: bool,
    ) -> RepomixConfig {
        let mut cfg = RepomixConfig::default();
        cfg.output.truncate_base64 = truncate_base64;
        cfg.output.remove_comments = remove_comments;
        cfg.output.remove_empty_lines = remove_empty;
        cfg
    }
}
