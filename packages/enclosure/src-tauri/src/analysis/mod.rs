use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tauri::ipc::{Request};
use crate::file::FileHandler;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpFileOptions {
    pub(crate) name: String,
    pub(crate) suffix: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub(crate) code: u16,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
    #[serde(rename = "fileProps")]
    pub(crate) file_props: HttpFileOptions,
    pub(crate) error: String,
    #[serde(rename = "imageSuffixes")]
    pub(crate) image_suffixes: String,
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
