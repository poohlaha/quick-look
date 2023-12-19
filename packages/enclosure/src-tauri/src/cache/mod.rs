//! cache

use crate::config::{FileProps, History, HISTORY_COUNT, HISTORY_FILE};
use crate::error::Error;
use crate::system::menu::FILE_RECENT_FILES_ID;
use crate::utils::file::FileUtils;
use log::info;
use uuid::Uuid;

pub struct Cache;

impl Cache {
    /// 读取历史记录
    pub fn read_history() -> Result<(String, Vec<History>), String> {
        let path = FileUtils::create_temp_dir("", false)?;
        let path = path.join(HISTORY_FILE);
        let file_path = path.as_path().to_string_lossy().to_string();
        info!("history file path: {}", &file_path);

        let mut contents: Vec<History> = Vec::new();

        if path.exists() {
            info!("history found ...");
            let content = FileUtils::read_file_string(&file_path)?;
            if !content.is_empty() {
                contents = serde_json::from_str(&content).map_err(|err| Error::Error(err.to_string()).to_string())?;
            }
        } else {
            info!("no history found ...");
        }

        Ok((file_path, contents))
    }

    /// 保存历史记录
    pub fn save_history(file_props: &FileProps) -> Result<(), String> {
        // 存储历史记录
        let (file_path, mut contents) = Cache::read_history()?;

        // 判断名字和路径是否已在存，如果不存在则添加
        let name = &file_props.name;
        let path = &file_props.path;
        let uuid = Uuid::new_v4().to_string();

        let mut has_found = false;
        if !contents.is_empty() {
            let found = contents.iter().find(|c| &c.name == name && &c.path == path);
            if found.is_some() {
                has_found = true
            }
        }

        // 已找到
        if has_found {
            info!("found history ...");
            return Ok(());
        }

        // 未找到
        info!("not found history ...");

        // 按时间倒序
        contents.sort_by(|a, b| b.update_time.cmp(&a.update_time));

        // 判断是不是条数大于 HISTORY_COUNT
        if contents.len() >= HISTORY_COUNT {
            contents = contents[..HISTORY_COUNT - 1].to_vec(); // 切掉 HISTORY_COUNT - 1 条
        }

        contents.push(History {
            id: format!("{}{}", FILE_RECENT_FILES_ID, uuid),
            name: name.to_string(),
            path: path.to_string(),
            update_time: chrono::Local::now().timestamp(),
        });

        // 写入 history
        let content = serde_json::to_string_pretty(&contents).unwrap();
        FileUtils::write_to_file_when_clear(&file_path, &content)?;
        return Ok(());
    }
}
