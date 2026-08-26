//! ClipLinux CLI.

use clap::{Parser, Subcommand};
use clipl_core::{paths, Capability, ClipboardItemId, PlatformAdapter, SupportLevel};
use clipl_platform::{capabilities_for, select_adapter, select_clipboard_backend, AdapterKind};
use clipl_protocol::{IpcClient, Request, Response};

#[derive(Parser)]
#[command(
    name = "clipl",
    version,
    about = "ClipLinux — universal paste, clipboard, and expression for Linux"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version information.
    Version,
    /// Probe session identity and capability support levels (no daemon required).
    Doctor {
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Ask the running daemon for status.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Ping the running daemon.
    Ping,
    /// Clipboard history (requires daemon).
    History {
        #[command(subcommand)]
        action: Option<HistoryCmd>,
        /// Maximum items when listing.
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        json: bool,
    },
    /// Show the desktop picker (requires daemon + running desktop).
    Open,
    /// Hide the desktop picker (requires a subscribed desktop process).
    Hide,
    /// Toggle the desktop picker (requires a subscribed desktop process).
    Toggle,
}

#[derive(Subcommand)]
enum HistoryCmd {
    /// List recent items (default).
    List {
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        json: bool,
    },
    /// Search stored text.
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        json: bool,
    },
    /// Delete one item by id.
    Delete { id: String },
    /// Remove unpinned items.
    Clear {
        #[arg(long)]
        yes: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = dispatch(cli.command) {
        eprintln!("clipl: {err}");
        std::process::exit(1);
    }
}

fn dispatch(command: Commands) -> Result<(), clipl_core::Error> {
    match command {
        Commands::Version => {
            println!("clipl {}", env!("CARGO_PKG_VERSION"));
            println!("protocol {}", clipl_protocol::PROTOCOL_VERSION);
        }
        Commands::Doctor { json } => doctor(json)?,
        Commands::Status { json } => status(json)?,
        Commands::Ping => ping()?,
        Commands::History {
            action,
            limit,
            json,
        } => history(action, limit, json)?,
        Commands::Open => desktop_action(clipl_protocol::Request::ShowDesktop)?,
        Commands::Hide => desktop_action(clipl_protocol::Request::HideDesktop)?,
        Commands::Toggle => desktop_action(clipl_protocol::Request::ToggleDesktop)?,
    }
    Ok(())
}

fn doctor(json: bool) -> Result<(), clipl_core::Error> {
    let adapter = select_adapter();
    let identity = adapter.identity();
    let cfg = load_config();
    let caps = capabilities_for(&identity, &cfg.clipboard);
    let preferred = AdapterKind::preferred(&identity);
    let selected = select_clipboard_backend(&identity, &cfg.clipboard);

    if json {
        let payload = serde_json::json!({
            "adapter": adapter.name(),
            "preferred_adapter": preferred.as_str(),
            "preferred_adapter_implemented": preferred.is_implemented(),
            "clipboard_backend": selected.name,
            "monitoring_reason": selected.reason,
            "identity": identity,
            "capabilities": caps,
            "socket": paths::socket_path(),
            "activation": clipl_platform::select_activation_backend(&identity, &cfg.activation).snapshot,
        });
        println!("{}", serde_json::to_string_pretty(&payload).expect("json"));
        return Ok(());
    }

    println!("ClipLinux doctor");
    println!("  adapter:            {}", adapter.name());
    println!(
        "  preferred adapter:  {} (implemented: {})",
        preferred.as_str(),
        preferred.is_implemented()
    );
    println!("  clipboard backend:  {}", selected.name);
    println!("  monitoring:         {}", format_level(selected.watch));
    println!("  reason:             {}", selected.reason);
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
    println!("  socket:             {}", paths::socket_path().display());
    println!();
    let activation = clipl_platform::select_activation_backend(&identity, &cfg.activation);
    print!(
        "{}",
        clipl_platform::format_activation_report(&identity, &activation, false)
    );
    println!("Capabilities:");
    for cap in Capability::all() {
        let level = caps.level(*cap);
        println!("  {:<22} {}", cap.as_str(), format_level(level));
    }
    Ok(())
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

fn load_config() -> clipl_core::ClipLinuxConfig {
    let path = paths::config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| clipl_core::ClipLinuxConfig::from_toml_str(&text).ok())
        .unwrap_or_default()
}

fn client() -> Result<IpcClient, clipl_core::Error> {
    IpcClient::connect()
}

fn ping() -> Result<(), clipl_core::Error> {
    match client()?.request(Request::Ping)? {
        Response::Pong => {
            println!("pong");
            Ok(())
        }
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected response: {other:?}"
        ))),
    }
}

fn desktop_action(request: Request) -> Result<(), clipl_core::Error> {
    match client()?.request(request)? {
        Response::DesktopRouted { delivered: true } => Ok(()),
        Response::DesktopRouted { delivered: false } => Err(clipl_core::Error::Io(
            "desktop picker is not running (start clipl-desktop, then retry)".into(),
        )),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected response: {other:?}"
        ))),
    }
}

fn status(json: bool) -> Result<(), clipl_core::Error> {
    match client()?.request(Request::GetStatus)? {
        Response::Status(status) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&status).expect("json"));
            } else {
                println!("ClipLinux daemon");
                println!("  version:     {}", status.version);
                println!("  protocol:    {}", status.protocol_version);
                println!("  session:     {:?}", status.session);
                println!("  desktop:     {:?}", status.desktop);
                println!("  backend:     {}", status.backend);
                println!("  monitoring:  {:?}", status.monitoring);
                if !status.monitoring_reason.is_empty() {
                    println!("  reason:      {}", status.monitoring_reason);
                }
                println!("  database:    {}", status.database);
                println!("  socket:      {}", status.socket_path);
                println!(
                    "  privacy:     {}",
                    if status.privacy_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                println!(
                    "  history:     enabled={} limit={}",
                    status.history_enabled, status.history_limit
                );
                println!();
                println!("Activation");
                println!("  session:     {:?}", status.activation.session);
                println!("  desktop:     {:?}", status.activation.desktop);
                println!("  shortcut:    {}", status.activation.shortcut);
                println!(
                    "  backend:     {} ({})",
                    status.activation.backend.as_str(),
                    status.activation.capability.as_str()
                );
                println!("  status:      {}", status.activation.status.as_str());
                println!(
                    "  desktop app: {}",
                    if status.activation.desktop_connected {
                        "connected"
                    } else {
                        "not running"
                    }
                );
                if !status.activation.reason.is_empty() {
                    println!("  reason:      {}", status.activation.reason);
                }
            }
            Ok(())
        }
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected response: {other:?}"
        ))),
    }
}

fn history(
    action: Option<HistoryCmd>,
    default_limit: u32,
    default_json: bool,
) -> Result<(), clipl_core::Error> {
    match action {
        None | Some(HistoryCmd::List { .. }) => {
            let (limit, json) = match action {
                Some(HistoryCmd::List { limit, json }) => (limit, json),
                _ => (default_limit, default_json),
            };
            list_history(limit, json)
        }
        Some(HistoryCmd::Search { query, limit, json }) => search_history(&query, limit, json),
        Some(HistoryCmd::Delete { id }) => delete_history(&id),
        Some(HistoryCmd::Clear { yes }) => clear_history(yes),
    }
}

fn list_history(limit: u32, json: bool) -> Result<(), clipl_core::Error> {
    match client()?.request(Request::GetHistory { limit })? {
        Response::History(items) => print_items(&items, json),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected response: {other:?}"
        ))),
    }
}

fn search_history(query: &str, limit: u32, json: bool) -> Result<(), clipl_core::Error> {
    match client()?.request(Request::SearchHistory {
        query: query.to_string(),
        limit,
    })? {
        Response::History(items) => print_items(&items, json),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected response: {other:?}"
        ))),
    }
}

fn delete_history(id: &str) -> Result<(), clipl_core::Error> {
    let item_id: ClipboardItemId = id.parse()?;
    match client()?.request(Request::DeleteItem { item_id })? {
        Response::Deleted { existed } => {
            if existed {
                println!("deleted {id}");
            } else {
                println!("not found: {id}");
            }
            Ok(())
        }
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected response: {other:?}"
        ))),
    }
}

fn clear_history(yes: bool) -> Result<(), clipl_core::Error> {
    if !yes {
        return Err(clipl_core::Error::Invalid(
            "refusing to clear history without --yes".into(),
        ));
    }
    match client()?.request(Request::ClearHistory)? {
        Response::Cleared { count } => {
            println!("cleared {count} unpinned item(s)");
            Ok(())
        }
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected response: {other:?}"
        ))),
    }
}

fn print_items(items: &[clipl_core::ClipboardItem], json: bool) -> Result<(), clipl_core::Error> {
    if json {
        let rows: Vec<_> = items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id.to_string(),
                    "created_at": item.created_at.as_millis(),
                    "pinned": item.pinned,
                    "preview": visible_preview(item),
                    "type": item.content.type_name(),
                    "hidden": !item.sensitive.is_empty(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows).expect("json"));
        return Ok(());
    }
    if items.is_empty() {
        println!("(no history)");
        return Ok(());
    }
    for item in items {
        let pin = if item.pinned { "*" } else { " " };
        println!("{pin} {}  {}", item.id, visible_preview(item));
    }
    Ok(())
}

fn visible_preview(item: &clipl_core::ClipboardItem) -> String {
    if item.sensitive.is_empty() {
        item.content.preview(80)
    } else {
        format!("[{} hidden]", item.content.type_name())
    }
}
