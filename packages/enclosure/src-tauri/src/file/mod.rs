//! 文件操作

use std::fs::File;
use std::io::Read;
use crate::error::Error;
use base64::Engine;
use log::info;
use serde_json::{Map, Value};
use tauri::http::HeaderMap;
use tauri::ipc::{InvokeBody, Request};
use crate::analysis::HttpResponse;


/// 图片后缀
const IMAGE_SUFFIXES: [&str; 11] = ["jpeg", "jpg", "png", "gif", "tiff", "tif", "webp", "ico", "heic", "svg", "bmp"];

/// 压缩包后缀
const ARCHIVE_SUFFIXES: [&str; 7] = ["zip", "gz", "tar", "rar", "7z", ".bz2", "xz"];

pub struct FileHandler;

impl FileHandler {

    /// 从 `headers` 头中获取文件名, 中文名是 encode 的, 需要 decode
    fn get_filename(headers: &HeaderMap)-> Result<String, String> {
        info!("headers: {:#?}", headers);
        let filename = headers.get("fileName");
        info!("filename: {:#?}", filename);
        if filename.is_none() {
            return Err(Error::Error("`fileName` not in headers !".to_string()).to_string());
        }

        let mut file_name = String::new();
        if let Some(filename) = filename {
            let name = filename.to_str().map_err(|err| Error::Error(err.to_string()).to_string())?;
            file_name = name.to_string();
        }

        // decode filename
        let file_name = urlencoding::decode(&file_name).map_err(|err| Error::Error(err.to_string()).to_string())?;
        let file_name = file_name.to_string();
        info!("filename decode: {:#?}", &file_name);
        return Ok(file_name)
    }

    /// 获取文件后缀
    fn get_file_suffix(file_name: &str) -> String {
        let names: Vec<&str> = file_name.split(".").collect();
        let mut file_suffix = String::new();
        if let Some(suffix) = names.last() {
            file_suffix = suffix.to_string()
        }

        return file_suffix
    }

    /// 通过文件流读取文件
    pub fn exec(request: Request) -> Result<HttpResponse, String> {
        let mut response = HttpResponse::default();
        response.image_suffixes = IMAGE_SUFFIXES.join(",").to_string();
        response.code = 500;

        // get filename in headers
        let file_name = Self::get_filename(request.headers())?;
        response.file_props.name = file_name.clone();

        // file suffix
        let file_suffix = Self::get_file_suffix(&file_name);
        response.file_props.suffix = file_suffix.clone();

        let body = request.body();
        if let InvokeBody::Raw(data) = body {
            // blob
            return Self::prepare_blob(data, response)
        } else if let InvokeBody::Json(data) = body {
            // json
            return Self::prepare_json(data, response)
        }

        return Ok(response);
    }

    /// 处理二进制数据
    fn prepare_blob(data: &Vec<u8>, mut response: HttpResponse) -> Result<HttpResponse, String> {
        info!("2");
        let suffix = response.file_props.suffix.clone();
        if IMAGE_SUFFIXES.contains(&suffix.as_str()) {
            // image suffix
            // 转成 base64
            let str = base64::engine::general_purpose::STANDARD.encode::<Vec<u8>>(data.clone());
            let mut content = String::from("data:image/png;base64,");
            content.push_str(&str);

            response.code = 200;
            response.body = content;
            info!("success");
            return Ok(response);
        } else if ARCHIVE_SUFFIXES.contains(&suffix.as_str()) {
            // archive suffix

            info!("success");
            return Ok(response);
        } else {
            let content = String::from_utf8(data.to_vec()).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;

            response.code = 200;
            response.body = content;
            info!("success");
            return Ok(response);
        }
    }

    /// 读取 json 数据
    fn prepare_json(data: &Value, response: HttpResponse) -> Result<HttpResponse, String> {
        let map = Map::new();
        let obj: &Map<String, Value> = data.as_object().unwrap_or(&map);
        let file_path = obj.get("filePath");
        if file_path.is_none() {
            return Err(Error::Error("`fileName` not in headers !".to_string()).to_string());
        }

        let file_path = file_path.unwrap().as_str().unwrap();
        info!("file path: {}", file_path);

        // 读取文件
        let content = Self::read_file(file_path)?;
        Self::prepare_blob(&content, response)
    }

    /// 读取文件
    fn read_file(file_path: &str) -> Result<Vec<u8>, String> {
        let mut file = File::open(&file_path).map_err(|err| {
            return Error::Error(err.to_string()).to_string();
        })?;

        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents).map_err(|err| {
            return Error::Error(err.to_string()).to_string();
        })?;

        Ok(contents)
    }

}