use crate::error::Error;
use base64::Engine;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tauri::ipc::{InvokeBody, Request};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpFileOptions {
    name: String,
    suffix: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(rename = "fileProps")]
    pub file_props: HttpFileOptions,
    pub error: String,
    #[serde(rename = "imageSuffixes")]
    pub image_suffixes: String,
}

const IMAGE_SUFFIXES: [&str; 11] = ["jpeg", "jpg", "png", "gif", "tiff", "tif", "webp", "ico", "heic", "svg", "bmp"];

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

#[tauri::command]
pub fn open_file(request: Request) -> Result<HttpResponse, String> {
    let mut response = HttpResponse::default();
    response.image_suffixes = IMAGE_SUFFIXES.join(",").to_string();
    response.code = 500;

    // get filename in headers
    let headers = request.headers();
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
    response.file_props.name = file_name.clone();
    info!("filename decode: {:#?}", file_name);

    // file suffix
    let names: Vec<&str> = file_name.split(".").collect();
    let mut file_suffix = String::new();
    if let Some(suffix) = names.last() {
        file_suffix = suffix.to_string()
    }

    response.file_props.suffix = file_suffix.clone();

    let body = request.body();
    if let InvokeBody::Raw(data) = body {
        // image suffix
        if IMAGE_SUFFIXES.contains(&file_suffix.as_str()) {
            // 转成 base64
            let str = base64::engine::general_purpose::STANDARD.encode::<Vec<u8>>(data.clone());
            let mut content = String::from("data:image/png;base64,");
            content.push_str(&str);

            response.code = 200;
            response.body = content;
            return Ok(response);
        } else {
            let content = String::from_utf8(data.to_vec()).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;

            response.code = 200;
            response.body = content;
            return Ok(response);
        }
    }

    return Ok(response);
}
