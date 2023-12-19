//! prepare trait

use crate::config::FileProps;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub trait HttpResponseData: Default + Debug + Clone + Serialize + DeserializeOwned + 'static {}

pub trait Prepare<R>
where
    R: HttpResponseData,
{
    // 通过 file_path 处理
    fn with_file_path(file_path: &str, response: R) -> Result<R, String> {
        Ok(response)
    }

    // 通过 file reader 处理
    fn with_file_reader(reader: BufReader<File>, response: R) -> Result<R, String> {
        Ok(response)
    }

    // 通过 response 处理
    fn with_response(response: R) -> Result<R, String> {
        Ok(response)
    }
}

pub trait Treat<R>
where
    R: HttpResponseData,
{
    fn handle(app: &tauri::AppHandle, request: tauri::ipc::Request) -> Result<R, String>;

    fn get_filename(headers: &tauri::http::HeaderMap) -> Result<String, String>;

    fn get_response(filename: &str) -> R;

    fn prepare(app: &tauri::AppHandle, body: &tauri::ipc::InvokeBody, response: R) -> Result<R, String>;

    fn prepare_blob(data: &Vec<u8>, response: R) -> Result<R, String>;

    fn prepare_json(file_path: &str, response: R) -> Result<R, String>;

    fn prepare_directory(path: &PathBuf, response: R) -> Result<R, String>;

    fn prepare_file(file_path: &str, response: R) -> Result<R, String>;

    fn compare_file(file_path: &str, response: R) -> Result<R, String>;

    fn prepare_file_props(file_path: &str) -> Result<FileProps, String>;

    fn prepare_response(response: R, content: Option<String>, _type: &str) -> R;
}
