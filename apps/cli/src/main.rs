//! UniPick CLI.

use clap::{Parser, Subcommand};
use unipick_core::{Capability, PlatformAdapter, SupportLevel};
use unipick_platform::{select_adapter, AdapterKind};
use unipick_protocol::{Envelope, Message, Request};

#[derive(Parser)]
#[command(
    name = "unipick",
    version,
    about = "UniPick — universal paste, clipboard, and expression for Linux"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version information.
    Version,
    /// Probe session identity and capability support levels.
    Doctor {
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Build a ping envelope (does not contact a daemon yet).
    Ping,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            println!("unipick {}", env!("CARGO_PKG_VERSION"));
            println!("UniPick foundation — daemon IPC is not wired yet.");
        }
        Commands::Doctor { json } => doctor(json),
        Commands::Ping => ping(),
    }
}

fn doctor(json: bool) {
    let adapter = select_adapter();
    let identity = adapter.identity();
    let caps = adapter.capabilities();
    let preferred = AdapterKind::preferred(&identity);

    if json {
        let payload = serde_json::json!({
            "adapter": adapter.name(),
            "preferred_adapter": preferred.as_str(),
            "preferred_adapter_implemented": preferred.is_implemented(),
            "identity": identity,
            "capabilities": caps,
        });
        println!("{}", serde_json::to_string_pretty(&payload).expect("json"));
        return;
    }

    println!("UniPick doctor");
    println!("  adapter:            {}", adapter.name());
    println!(
        "  preferred adapter:  {} (implemented: {})",
        preferred.as_str(),
        preferred.is_implemented()
    );
    println!("  platform:           {:?}", identity.platform);
    println!("  session:            {:?}", identity.session);
    println!("  desktop:            {:?}", identity.desktop);
    println!(
        "  XDG_SESSION_TYPE:    {}",
        identity.xdg_session_type.as_deref().unwrap_or("(unset)")
    );
    println!(
        "  XDG_CURRENT_DESKTOP: {}",
        identity.xdg_current_desktop.as_deref().unwrap_or("(unset)")
    );
    println!();
    println!("Capabilities (Unknown means not probed; this is not a failure):");
    for cap in Capability::all() {
        let level = caps.level(*cap);
        println!("  {:<22} {}", cap.as_str(), format_level(level));
    }
    println!();
    println!("Clipboard monitoring is not implemented in the foundation.");
}

fn format_level(level: SupportLevel) -> &'static str {
    match level {
        SupportLevel::Native => "native",
        SupportLevel::Portal => "portal",
        SupportLevel::Fallback => "fallback",
        SupportLevel::Unsupported => "unsupported",
        SupportLevel::Unknown => "unknown",
        _ => "unknown",
    }
}

fn ping() {
    let envelope = Envelope::new(Message::Request(Request::Ping));
    let bytes = envelope.to_json_bytes().expect("serialize ping");
    println!("ping envelope (not sent; daemon IPC is not implemented):");
    println!("{}", String::from_utf8_lossy(&bytes));
}
