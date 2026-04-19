mod cli;

use miette::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().or_else(|_| EnvFilter::try_new("info")).unwrap())
        .init();

    cli::run()?;

    Ok(())
}
