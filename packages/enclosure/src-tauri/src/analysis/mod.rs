mod archive;
mod document;
mod excel;
pub mod process;

use crate::analysis::archive::Archive;
use crate::analysis::process::Process;
use crate::config::HttpResponse;
use tauri::ipc::Request;

/// 通过文件流或文件路径读取文件
#[tauri::command]
pub async fn process<'a>(app: tauri::AppHandle, request: Request<'a>) -> Result<HttpResponse, String> {
    Process::handle(&app, request).await
}

/// 解压压缩包
#[tauri::command]
pub fn unarchive(file_path: &str, full_path: &str) -> Result<HttpResponse, String> {
    Archive::unarchive(file_path, full_path)
}
