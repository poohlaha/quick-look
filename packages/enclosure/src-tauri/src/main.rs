// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod archive;
mod error;
mod preview;
mod process;
mod utils;

use analysis::{file_handler, read_file_association, unarchive};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![file_handler, unarchive])
        .run(tauri::generate_context!())
        .expect("error while running QuickLook application");

    // Trigger `application:openURLs:`
    read_file_association();
}
