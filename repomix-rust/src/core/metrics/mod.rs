use anyhow::Result;
use std::sync::{Arc, OnceLock};
use tiktoken_rs::CoreBPE;

// Static caches for BPE encoders (initialized once, reused across all calls)
static O200K: OnceLock<Arc<CoreBPE>> = OnceLock::new();
static CL100K: OnceLock<Arc<CoreBPE>> = OnceLock::new();
static P50K: OnceLock<Arc<CoreBPE>> = OnceLock::new();
static R50K: OnceLock<Arc<CoreBPE>> = OnceLock::new();

/// Get or initialize the BPE encoder for the specified encoding.
/// The encoder is cached and reused across all calls.
fn bpe_for(encoding: &str) -> Arc<CoreBPE> {
    match encoding {
        "o200k_base" => O200K
            .get_or_init(|| Arc::new(tiktoken_rs::o200k_base().expect("Failed to load o200k_base")))
            .clone(),
        "cl100k_base" => CL100K
            .get_or_init(|| Arc::new(tiktoken_rs::cl100k_base().expect("Failed to load cl100k_base")))
            .clone(),
        "p50k_base" => P50K
            .get_or_init(|| Arc::new(tiktoken_rs::p50k_base().expect("Failed to load p50k_base")))
            .clone(),
        "r50k_base" => R50K
            .get_or_init(|| Arc::new(tiktoken_rs::r50k_base().expect("Failed to load r50k_base")))
            .clone(),
        _ => {
            tracing::warn!("Unknown encoding: {}, falling back to o200k_base", encoding);
            O200K
                .get_or_init(|| Arc::new(tiktoken_rs::o200k_base().expect("Failed to load o200k_base")))
                .clone()
        }
    }
}

/// Count tokens in the given content using the specified encoding.
/// 
/// Uses `encode_ordinary` instead of `encode_with_special_tokens` for better performance,
/// matching the Node.js version's approach (special tokens are treated as ordinary text).
pub fn count_tokens(content: &str, encoding: &str) -> Result<usize> {
    let bpe = bpe_for(encoding);
    // Use encode_ordinary for performance (matches Node.js version behavior)
    let tokens = bpe.encode_ordinary(content);
    Ok(tokens.len())
}

/// Count tokens for multiple texts in batch.
/// 
/// This is more efficient than calling `count_tokens` multiple times
/// as it reuses the same BPE encoder instance.
pub fn count_tokens_batch<'a, I>(texts: I, encoding: &str) -> Result<Vec<usize>>
where
    I: IntoIterator<Item = &'a str>,
{
    let bpe = bpe_for(encoding);
    let results: Vec<usize> = texts
        .into_iter()
        .map(|text| bpe.encode_ordinary(text).len())
        .collect();
    Ok(results)
}
