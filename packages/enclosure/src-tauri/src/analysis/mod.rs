mod archive;
mod document;
pub mod process;

use crate::analysis::archive::Archive;
use crate::analysis::process::Process;
use crate::config::HttpResponse;
use crate::prepare::Treat;
use tauri::ipc::Request;

/// 通过文件流或文件路径读取文件
#[tauri::command]
pub fn process(app: tauri::AppHandle, request: Request) -> Result<HttpResponse, String> {
    Process::handle(&app, request)
}

/// 解压压缩包
#[tauri::command]
pub fn unarchive(file_path: &str, full_path: &str) -> Result<HttpResponse, String> {
    Archive::unarchive(file_path, full_path)
}
