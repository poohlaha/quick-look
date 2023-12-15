// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod archive;
mod error;
mod preview;
mod process;
mod utils;

use tauri::Manager;
use tauri::tray::{ClickType, TrayIconBuilder};
use analysis::{file_handler, read_file_association, unarchive};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(move |app| {
            let _ = TrayIconBuilder::with_id("quick-look-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .on_tray_icon_event(|tray, event| {
                    if event.click_type == ClickType::Left {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }).build(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![file_handler, unarchive])
        .run(tauri::generate_context!())
        .expect("error while running QuickLook application");

    // Trigger `application:openURLs:`
    read_file_association();
}
