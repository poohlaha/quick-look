use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tauri::ipc::{Request};
use crate::file::{FileHandler};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SuffixProps {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) _type: String,
    pub(crate) list: Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileProps {
    pub key: String,
    pub name: String,
    pub suffix: String,
    pub prefix: String,
    pub path: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    pub kind: String,
    pub size: String,
    pub old_size: u64,
    pub packed: String,
    pub modified: String,
    pub permissions: String,
    pub executable: bool,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    pub files: Vec<FileProps>
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub(crate) code: u16,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
    #[serde(rename = "fileProps")]
    pub(crate) file_props: FileProps,
    pub(crate) error: String,
    #[serde(rename = "suffixProps")]
    pub(crate) suffix_props: SuffixProps,
}


/// 读取关联的文件
pub fn read_file_association() {
    info!("read file association ...");
    let args: Vec<String> = env::args().collect();
    info!("args: {:#?}", args);
    if args.len() > 1 {
        let file_path = &args[1];
        info!("file path: {}", file_path);
    }
}

/// 通过文件流或文件路径读取文件
#[tauri::command]
pub fn file_handler(request: Request) -> Result<HttpResponse, String> {
    FileHandler::exec(request)
}
