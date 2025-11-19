use tracing_subscriber::{fmt, EnvFilter};

pub fn init() {
    // Use RUST_LOG environment variable if present, otherwise default to info
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false) // Do not show module path (to match Repomix style)
        .with_timer(fmt::time::SystemTime)
        .init();
}
