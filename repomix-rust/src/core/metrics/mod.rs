use anyhow::Result;
use tiktoken_rs::CoreBPE;

pub fn count_tokens(content: &str, encoding: &str) -> Result<usize> {
    // Map encoding string to tiktoken encoding
    // For now, we support o200k_base (gpt-4o) and cl100k_base (gpt-4)
    // If unknown, default to cl100k_base or error?
    // tiktoken-rs provides `get_bpe_from_model` or `get_bpe_from_tokenizer`.
    
    // Let's try to match the encoding string to what tiktoken-rs expects.
    // The config default is "o200k_base".
    
    let bpe = match encoding {
        "o200k_base" => tiktoken_rs::o200k_base().unwrap(),
        "cl100k_base" => tiktoken_rs::cl100k_base().unwrap(),
        "p50k_base" => tiktoken_rs::p50k_base().unwrap(),
        "r50k_base" => tiktoken_rs::r50k_base().unwrap(),
        _ => {
            // Fallback or error. Let's try cl100k_base as safe default or try to look up by name if possible.
            // tiktoken-rs doesn't seem to have a dynamic lookup by string easily exposed without model name.
            // But we can try to map common names.
            tracing::warn!("Unknown encoding: {}, falling back to o200k_base", encoding);
            tiktoken_rs::o200k_base().unwrap()
        }
    };

    let tokens = bpe.encode_with_special_tokens(content);
    Ok(tokens.len())
}
