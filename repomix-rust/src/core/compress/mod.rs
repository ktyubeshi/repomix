use anyhow::Result;
use tree_sitter::{Parser, Query, QueryCursor};
use std::collections::HashMap;

mod queries;

const CHUNK_SEPARATOR: &str = "⋮----";

struct Chunk {
    content: String,
    start_row: usize,
    end_row: usize,
}

pub fn compress_content(content: &str, extension: &str) -> Result<String> {
    let (language, query_str) = match extension {
        "rs" => (tree_sitter_rust::language(), queries::QUERY_RUST),
        "ts" | "tsx" => (tree_sitter_typescript::language_typescript(), queries::QUERY_TYPESCRIPT),
        "js" | "jsx" => (tree_sitter_javascript::language(), queries::QUERY_JAVASCRIPT),
        "py" => (tree_sitter_python::language(), queries::QUERY_PYTHON),
        "go" => (tree_sitter_go::language(), queries::QUERY_GO),
        _ => return Ok(content.to_string()),
    };

    let mut parser = Parser::new();
    parser.set_language(language)?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse code"))?;

    let query = Query::new(language, query_str)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let lines: Vec<&str> = content.lines().collect();
    let mut captured_chunks: Vec<Chunk> = Vec::new();

    for m in matches {
        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize].as_str();
            
            let is_name = capture_name.contains("name");
            let is_comment = capture_name.contains("comment");
            let is_import = capture_name.contains("import") || capture_name.contains("require");
            
            if is_name || is_comment || is_import {
                let start_row = capture.node.start_position().row;
                let end_row = capture.node.end_position().row;
                
                if start_row >= lines.len() {
                    continue;
                }
                
                // Ensure end_row is within bounds
                let actual_end_row = if end_row < lines.len() { end_row } else { lines.len() - 1 };
                
                if start_row > actual_end_row {
                     continue;
                }

                let selected_lines = &lines[start_row..=actual_end_row];
                let chunk_content = selected_lines.join("\n");
                
                captured_chunks.push(Chunk {
                    content: chunk_content,
                    start_row,
                    end_row: actual_end_row,
                });
            }
        }
    }

    // Filter duplicated chunks (keep longest for same start row)
    let mut chunks_by_start_row: HashMap<usize, Vec<Chunk>> = HashMap::new();
    for chunk in captured_chunks {
        chunks_by_start_row.entry(chunk.start_row).or_default().push(chunk);
    }

    let mut filtered_chunks: Vec<Chunk> = Vec::new();
    for (_, mut row_chunks) in chunks_by_start_row {
        row_chunks.sort_by(|a, b| b.content.len().cmp(&a.content.len()));
        if let Some(best_chunk) = row_chunks.into_iter().next() {
            filtered_chunks.push(best_chunk);
        }
    }
    
    filtered_chunks.sort_by(|a, b| a.start_row.cmp(&b.start_row));

    // Merge adjacent chunks
    if filtered_chunks.is_empty() {
        return Ok(String::new());
    }

    let mut merged_chunks: Vec<Chunk> = Vec::new();
    let mut current_chunk = filtered_chunks[0].content.clone();
    let mut current_start = filtered_chunks[0].start_row;
    let mut current_end = filtered_chunks[0].end_row;

    for i in 1..filtered_chunks.len() {
        let next_chunk = &filtered_chunks[i];
        if current_end + 1 == next_chunk.start_row {
            current_chunk.push('\n');
            current_chunk.push_str(&next_chunk.content);
            current_end = next_chunk.end_row;
        } else {
            merged_chunks.push(Chunk {
                content: current_chunk,
                start_row: current_start,
                end_row: current_end,
            });
            current_chunk = next_chunk.content.clone();
            current_start = next_chunk.start_row;
            current_end = next_chunk.end_row;
        }
    }
    merged_chunks.push(Chunk {
        content: current_chunk,
        start_row: current_start,
        end_row: current_end,
    });

    let final_content = merged_chunks
        .iter()
        .map(|c| c.content.trim())
        .collect::<Vec<&str>>()
        .join(&format!("\n{}\n", CHUNK_SEPARATOR));

    Ok(final_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_rust_simple() {
        let content = r#"
// This is a comment
fn main() {
    println!("Hello");
    let x = 1;
}

fn other() {
    // body
}
"#;
        let compressed = compress_content(content, "rs").unwrap();
        
        assert!(compressed.contains("fn main() {"));
        // println! is a macro invocation, so it is preserved
        assert!(compressed.contains("println!")); 
        // let statement is not captured, so it should be removed
        assert!(!compressed.contains("let x = 1;"));
        
        assert!(compressed.contains("fn other() {"));
        assert!(compressed.contains("⋮----"));
        assert!(compressed.contains("// This is a comment"));
    }

    #[test]
    fn test_compress_python_simple() {
        let content = r#"
def my_func():
    print("Hello")

class MyClass:
    def method(self):
        pass
"#;
        let compressed = compress_content(content, "py").unwrap();
        
        assert!(compressed.contains("def my_func():"));
        // print is a call, preserved
        assert!(compressed.contains("print(\"Hello\")"));
        // pass is not captured, removed
        assert!(!compressed.contains("pass"));
        
        assert!(compressed.contains("class MyClass:"));
        assert!(compressed.contains("def method(self):"));
    }
}
