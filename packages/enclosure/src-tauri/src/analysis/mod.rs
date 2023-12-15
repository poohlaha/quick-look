use crate::error::Error;
use crate::process::Process;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tauri::ipc::Request;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SuffixProps {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub list: Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileProps {
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
    pub files: Vec<FileProps>,
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub id: String,
    pub name: String,
    pub path: String,
    pub update_time: i64
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
pub fn file_handler(app: tauri::AppHandle, request: Request) -> Result<HttpResponse, String> {
    Process::exec(&app, request)
}

/// 解压压缩包
#[tauri::command]
pub fn unarchive(file_path: &str, full_path: &str) -> Result<HttpResponse, String> {
    let mut response = HttpResponse::default();
    let unarchive_path = Path::new(file_path);
    let archive_path = Path::new(full_path);

    if !unarchive_path.exists() || !archive_path.exists() {
        response.error = "文件解压失败, 路径不存在!".to_string();
        return Ok(response);
    }

    if unarchive_path.is_dir() {
        response.error = format!("{} is a directory !", file_path);
        return Ok(response);
    }

    let download_path = unarchive_path.parent();
    if download_path.is_none() {
        response.error = "文件解压失败, 父路径不存在!".to_string();
        return Ok(response);
    }

    let download_path = download_path.unwrap();
    if !download_path.exists() {
        response.error = "文件解压失败, 父路径不存在!".to_string();
        return Ok(response);
    }

    // 拷贝目录
    info!("unarchive copy files ...");
    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.overwrite = true;
    let download_path = download_path.to_string_lossy().to_string();
    fs_extra::copy_items(&[full_path], &download_path, &copy_options).map_err(|err| Error::Error(err.to_string()).to_string())?;

    response.code = 200;
    info!("unarchive success!");
    Ok(response)
}
