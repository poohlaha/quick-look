//! 系统菜单

use tauri::menu::{AboutMetadata, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Wry};
pub struct Menu;

const FILE_RECENT_FILES: &str = "__file_recent_files__";
impl Menu {

    /// 创建系统菜单, 参考 tauri/menu/menu.rs
    pub fn create_system_menus(app: &AppHandle<Wry>) -> tauri::Result<tauri::menu::Menu<Wry>> {
        let pkg_info = app.package_info();
        let config = app.config();

        let icon = tauri::Icon::Raw(include_bytes!("../../icons/Square150x150Logo.png").to_vec());

        // about
        let about_metadata = AboutMetadata {
            name: Some(pkg_info.name.clone()),
            version: Some(pkg_info.version.to_string()),
            copyright: config.tauri.bundle.copyright.clone(),
            authors: config.tauri.bundle.publisher.clone().map(|p| vec![p]),
            comments: Some(pkg_info.description.to_string()),
            license: Some("MIT/Apache-2.0".to_string()),
            website: Some("https://github.com/poohlaha/quick-look".to_string()),
            icon: Some(icon),
            ..Default::default()
        };

        // mac 名字对应菜单
        let name_menu_items = Submenu::with_items(
            app,
            pkg_info.name.clone(),
            true,
            &[
                &PredefinedMenuItem::about(app, Some("About"), Some(about_metadata)), // about
                &PredefinedMenuItem::separator(app),
                &PredefinedMenuItem::quit(app, Some("Quit")), // quit
            ],
        )?;

        // File
        let file_menu_items = Self::get_file_menus(app)?;

        // Help
        let help_menu_items = Submenu::with_id_and_items(
            app,
            tauri::menu::HELP_SUBMENU_ID,
            "Help",
            true,
            &[
                #[cfg(not(target_os = "macos"))]
                &PredefinedMenuItem::about(app, Some("About"), Some(about_metadata.clone())),
            ],
        )?;

        let menu = tauri::menu::Menu::with_items(
            app,
            &[
                #[cfg(target_os = "macos")]
                &name_menu_items, // mac

                #[cfg(not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
                )))]
                &file_menu_items,

                #[cfg(not(target_os = "macos"))]
                &help_menu_items,
            ]
        );

        menu
    }

    // 获取 File 菜单及其子菜单
    fn get_file_menus(app: &AppHandle<Wry>) -> tauri::Result<Submenu<Wry>> {
        let recent_files_menu = Submenu::with_id_and_items(
            app,
            FILE_RECENT_FILES,
            "Recent Files",
            true,
            &[]
        )?;

        // File
        Submenu::with_items(
            app,
            "File",
            true,
            &[
                &recent_files_menu
            ],
        )
    }
}