//! 系统菜单

use std::collections::HashSet;
use log::{error, info};
use tauri::menu::{AboutMetadata, MenuEvent, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, Wry};
use crate::analysis::{History};
use crate::process::{Process};

pub struct Menu;

pub const FILE_RECENT_FILES: &str = "__FILE_HISTORY__";
const FILE_RECENT_NO_DATA: &str = "__FILE_HISTORY__NO_DATA_";

const NO_DATA: &str = "No Data";

const MENUS: [&str; 1] = [
    "History"
];

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

        // History
        let history_menu_items = Self::get_history_menus(app)?;

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
                &history_menu_items,

                #[cfg(not(target_os = "macos"))]
                &help_menu_items,
            ]
        );

        menu
    }


    /// 获取历史记录子菜单
    fn get_history_submenus(app: &AppHandle, recent_history_menu: &mut Submenu<Wry>, contents: &Vec<History>) {
        // 查找是否有相同的名字，如果是相同名字，则 name 用 path 代替
        let mut submenu_set = HashSet::new();
        let mut submenu_contents: Vec<History> = Vec::new();
        for content in contents.iter() {
            if submenu_set.insert(content.name.clone()) {
                submenu_contents.push(content.clone())
            } else {
                let mut new_content = content.clone();
                new_content.name = new_content.path.clone(); // name 取 path
                submenu_contents.push(new_content)
            }
        }

        // 根据历史记录创建菜单
        for content in submenu_contents.iter() {
            let menu = MenuItem::with_id(
                app,
                &content.id,
                &content.name,
                true,
                None
            );

            recent_history_menu.append(&menu).unwrap();
        }
    }

    // 获取 History 菜单及其子菜单
    fn get_history_menus(app: &AppHandle<Wry>) -> tauri::Result<Submenu<Wry>> {
        let mut menu = Submenu::with_id_and_items(
            app,
            FILE_RECENT_FILES,
            MENUS.get(0).unwrap(),
            true,
            &[]
        )?;

        // 读取 HISTORY 文件
        let (_, contents) = Process::read_history().unwrap();
        if contents.is_empty() {
            menu.append(&Self::create_history_no_data(app)).unwrap();
        } else {
            Self::get_history_submenus(app, &mut menu, &contents);
        }

        Ok(menu)
    }

    /// 获取 History Menu
    fn get_history_item(app: &AppHandle) -> Option<Submenu<Wry>> {
        let menus = app.menu();
        if menus.is_none() {
            error!("get application menus error !");
            return None;
        }

        let menus = menus.unwrap();
        let items = menus.items();
        if items.is_err() {
            error!("get application menu items error: {}", items.err().unwrap().to_string());
            return None;
        }

        let items = items.unwrap();
        for item in items {
            if let MenuItemKind::Submenu(menu_item) = item {
                let menu_text = menu_item.text().unwrap();
                info!("sub menu text: {:#?}", &menu_text);
                let text = MENUS.get(0).unwrap();
                if &menu_text != text {
                    continue
                }

                // History Menu
                return Some(menu_item)
            }
        }

        None
    }

    // History no data
    fn create_history_no_data (app: &AppHandle) -> MenuItem<Wry> {
        MenuItem::with_id(
            app,
            format!("{}{}", FILE_RECENT_FILES, FILE_RECENT_NO_DATA),
            NO_DATA,
            true,
            None
        )
    }

    /// 清空历史记录子菜单
    fn clear_history_submenus(app: &AppHandle, ids: &Vec<String>) {
        if ids.is_empty() {
            info!("clear submenus ids is empty !");
            return
        }

        let file_menu = Self::get_history_item(app);
        if let Some(file_menu) = file_menu {
            let file_items = file_menu.items();
            if let Ok(file_items) = file_items {
                // remove sub menu
                for id in ids.iter() {
                    let remove_item = file_items.iter().find(|item| {
                        if let MenuItemKind::MenuItem(item) = item {
                            info!("sub menu item id: {:#?}", item.id())
                        }

                        return item.id() == id;
                    });

                    if remove_item.is_none() {
                        continue
                    }

                    let remove_item = remove_item.unwrap();
                    match file_menu.remove(remove_item) {
                        Ok(_) => {
                            error!("remove menu `{}` success !", id)
                        }
                        Err(err) => {
                            error!("remove menu `{}` error: {}", id, err.to_string())
                        }
                    }
                }

                // 判断是否有子菜单
                if let Ok(item) = file_menu.items() {
                    if item.is_empty() {
                        file_menu.append(&Self::create_history_no_data(app)).unwrap();
                    }
                }
            }
        }
    }

    /// 更新 History submenus
    pub fn update_history_submenus(app: &AppHandle) {
        info!("update `File` `Submenus` !");
        let file_menu = Self::get_history_item(app);
        if file_menu.is_none() {
            return
        }

        let mut file_menu = file_menu.unwrap();
        let file_items = file_menu.items();
        if let Ok(file_items) = file_items {
            let (_, contents) = Process::read_history().unwrap();
            if contents.is_empty() {
                info!("read history, content is empty, remove all `File` `Submenus` !");
                // 移除不存在的菜单
                info!("remove all `File` `Submenus` ...");
                for file_item in file_items.iter() {
                    match file_menu.remove(file_item) {
                        Ok(_) => {
                            error!("remove menu `{}` success !", file_item.id().0)
                        }
                        Err(err) => {
                            error!("remove menu `{}` error: {}", file_item.id().0, err.to_string())
                        }
                    }
                }
                return
            }

            // 移除不存在的菜单和 No Data
            info!("remove no data and not exists menus ...");
            let mut id_contents: Vec<String> = Vec::new();
            for file_item in file_items.iter() {
                let mut has_found = false;
                if let MenuItemKind::Submenu(menu_item) = file_item {
                    let menu_text = menu_item.text().unwrap();
                    if &menu_text == NO_DATA {
                        has_found = true
                    }
                } else {
                    let found = contents.iter().find(|c| &c.id == &file_item.id().0);
                    if found.is_some() {
                        has_found = true
                    } else {
                        id_contents.push(file_item.id().0.clone());
                    }
                }

                if !has_found {
                    continue
                }

                match file_menu.remove(file_item) {
                    Ok(_) => {
                        error!("remove menu `{}` success !", file_item.id().0)
                    }
                    Err(err) => {
                        error!("remove menu `{}` error: {}", file_item.id().0, err.to_string())
                    }
                }
            }

            info!("add new menus ...");
            let add_contents = contents.iter().filter_map(|c| {
                if id_contents.contains(&c.id) {
                    None
                } else {
                    Some(c.clone())
                }
            }).collect();

            // 重新设置菜单
            Self::get_history_submenus(app, &mut file_menu, &add_contents);
        }
    }

    /// 菜单点击事件
    pub fn on_menu_item_click(app: &AppHandle, event: &MenuEvent) {
        let event_id = &event.id.0;
        info!("event_id: {}", event_id);

        // Recent Files Sub Menus
        if event_id.starts_with(FILE_RECENT_FILES) {
            // 读取历史记录, 获取 path
            let (_, contents) = Process::read_history().unwrap();
            if contents.is_empty() {
                info!("can not found history !");
                // 清空菜单
                Self::clear_history_submenus(app, &vec![event_id.to_string()]);
                return
            }

            // 根据id 查找路径
            let content = contents.iter().find(|c| c.id.as_str() == event_id.as_str());
            if let Some(content) = content {
                let path = &content.path;
                let name = &content.name;
                if path.is_empty() || name.is_empty() {
                    info!("found history, path or name is empty !");
                    Self::clear_history_submenus(app, &vec![event_id.to_string()]);
                    return
                }

                // 根据路径查找数据
                let response = Process::exec_by_file_path(name, path);
                match response {
                    Ok(response) => {
                        let emit = app.emit("send_res_to_window", response);
                        match emit {
                            Ok(_) => {
                                info!("send result to window success !");
                            }
                            Err(err) => {
                                error!("send result to window error: {}", err.to_string());
                            }
                        }
                    }
                    Err(_) => {}
                }
            } else {
                info!("can not found history by `{}` !", &event_id);
                Self::clear_history_submenus(app, &vec![event_id.to_string()]);
            }
        }
    }
}