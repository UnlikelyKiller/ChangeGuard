use changeguard::cli;
use miette::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Build the log filter based on the verbose flag.
/// Stub: always returns the default "info" filter — tests for suppression will fail.
fn build_log_filter(_verbose: bool) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
}

fn run() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(build_log_filter(false))
        .init();

    cli::run()?;

    Ok(())
}

fn main() {
    // Windows debug builds with many clap subcommands can overflow the default
    // 1 MiB stack. Run the application logic in a thread with a larger stack.
    let result = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(run)
        .expect("Failed to spawn main thread")
        .join()
        .expect("Main thread panicked");

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_has_verbose_flag() {
        let args =
            changeguard::cli::Cli::try_parse_from(["changeguard", "--verbose", "doctor"]).unwrap();
        assert!(args.verbose);
        let args_short =
            changeguard::cli::Cli::try_parse_from(["changeguard", "-v", "doctor"]).unwrap();
        assert!(args_short.verbose);
    }

    #[test]
    fn cli_verbose_default_is_false() {
        let args = changeguard::cli::Cli::try_parse_from(["changeguard", "doctor"]).unwrap();
        assert!(!args.verbose);
    }

    #[test]
    fn build_log_filter_verbose_does_not_panic() {
        let _f = build_log_filter(true);
    }

    #[test]
    fn build_log_filter_quiet_does_not_panic() {
        let _f = build_log_filter(false);
    }
}
