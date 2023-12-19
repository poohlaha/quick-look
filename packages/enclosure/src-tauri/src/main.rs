// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod cache;
mod config;
mod error;
mod prepare;
mod system;
mod utils;

use crate::system::tray::Tray;
use analysis::{process, unarchive};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(move |app| {
            // 创建系统托盘
            Tray::builder(app);

            // 此处有问题，需要监听 app delegate 事件
            app.once_global("application:openURLs", move |event| {
                println!("application:openURLs : {:#?}", event);
            });

            // 菜单点击事件
            system::menu::Menu::on_menu_click(app);

            Ok(())
        })
        .menu(system::menu::Menu::create_system_menus)
        .invoke_handler(tauri::generate_handler![process, unarchive])
        .run(tauri::generate_context!())
        .expect("error while running `QuickLook` application");

    // Trigger `application:openURLs:`
}
