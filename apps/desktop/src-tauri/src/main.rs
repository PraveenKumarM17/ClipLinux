//! ClipLinux desktop entrypoint (foundation stub).

fn main() {
    println!(
        "{} {}",
        clipl_desktop_lib::APP_NAME,
        env!("CARGO_PKG_VERSION")
    );
    println!("app id: {}", clipl_desktop_lib::APP_ID);
    println!();
    println!("This binary is a compile-time placeholder.");
    println!("The production shell is Tauri v2 hosting a Svelte 5 UI.");
    println!("Run the frontend with the Tauri CLI after the desktop-shell milestone.");
}
