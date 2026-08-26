//! ClipLinux daemon entrypoint.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::Parser;
use clipl_daemon::{diagnostic_report, load_config, run};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "clipl-daemon", version, about = "ClipLinux background daemon")]
struct Args {
    /// Print session, backend, and database info, then exit.
    #[arg(long)]
    diagnose: bool,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("clipl-daemon: {err}");
            std::process::exit(2);
        }
    };

    if args.diagnose {
        print!("{}", diagnostic_report(&config));
        return;
    }

    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);
    if let Err(err) = ctrlc::set_handler(move || {
        flag.store(true, Ordering::SeqCst);
    }) {
        tracing::warn!("ctrlc handler: {err}");
    }

    if let Err(err) = run(config, shutdown) {
        eprintln!("clipl-daemon: {err}");
        std::process::exit(1);
    }
}
