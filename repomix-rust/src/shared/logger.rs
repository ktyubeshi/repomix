use tracing_subscriber::{fmt, EnvFilter};

use crate::cli::Cli;

pub fn init(args: &Cli) {
    // Use RUST_LOG environment variable if present, otherwise use args
    let default_filter = if args.verbose { "debug" } else { "info" };
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false) // Do not show module path (to match Repomix style)
        .with_timer(fmt::time::SystemTime)
        .init();
}
