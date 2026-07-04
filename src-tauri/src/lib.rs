// ============================================
// RajbariIT Remote Lite v1.0 — Library Entry
// ============================================
// Registers all Tauri commands from feature modules.

mod discovery;
mod screen;
mod input;
mod security;
mod transfer;

/// Run the Tauri application with all plugins and commands registered.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Discovery
            discovery::discover_devices,
            discovery::start_broadcasting,
            discovery::stop_broadcasting,
            discovery::get_device_name,
            // Screen
            screen::start_screen_host,
            screen::stop_screen_host,
            screen::capture_frame,
            screen::connect_to_host,
            // Input
            input::send_mouse_event,
            input::send_key_event,
            // Security
            security::generate_pin,
            security::get_current_pin,
            security::validate_pin,
            // Transfer
            transfer::send_file,
            transfer::start_receive_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RajbariIT Remote");
}
