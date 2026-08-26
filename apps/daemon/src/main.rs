//! UniPick daemon entrypoint.
//!
//! Foundation only: probe the session, load default privacy rules, and exit
//! without watching the clipboard.

use clap::Parser;
use unipick_core::{PlatformAdapter, SupportLevel};
use unipick_platform::{select_adapter, AdapterKind};
use unipick_privacy::default_rules;

#[derive(Parser)]
#[command(
    name = "unipick-daemon",
    version,
    about = "UniPick background daemon (foundation stub)"
)]
struct Args {
    /// Print the capability matrix and exit.
    #[arg(long)]
    diagnose: bool,
}

fn main() {
    let args = Args::parse();
    let adapter = select_adapter();
    let identity = adapter.identity();
    let caps = adapter.capabilities();
    let preferred = AdapterKind::preferred(&identity);
    let rules = default_rules();

    println!("unipick-daemon {}", env!("CARGO_PKG_VERSION"));
    println!("session: {:?} / {:?}", identity.session, identity.desktop);
    println!(
        "preferred adapter: {} (implemented: {})",
        preferred.as_str(),
        preferred.is_implemented()
    );
    println!("privacy rules loaded: {}", rules.len());
    let watch = caps.level(unipick_core::Capability::ClipboardWatch);
    let watch_label = match watch {
        SupportLevel::Unknown => "unknown (not probed)".to_string(),
        other => format!("{other:?}"),
    };
    println!("clipboard-watch: {watch_label}");

    if args.diagnose {
        println!("identity: {identity:?}");
        println!("capabilities: {caps:?}");
    }

    println!();
    println!("Foundation stub: clipboard monitoring is intentionally not started.");
    println!("The daemon will listen for IPC in a later milestone.");
}
