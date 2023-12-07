use crate::error::Error;
use base64::Engine;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::ipc::{InvokeBody, Request};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub error: String,

    #[serde(rename = "imageSuffixes")]
    pub image_suffixes: String,
}

const IMAGE_SUFFIXES: [&str; 11] = ["jpeg", "jpg", "png", "gif", "tiff", "tif", "webp", "ico", "heic", "svg", "bmp"];

#[tauri::command]
pub fn open_file(request: Request) -> Result<HttpResponse, String> {
    info!("request: {:#?}", request);
    let mut response = HttpResponse::default();
    response.image_suffixes = IMAGE_SUFFIXES.join(",").to_string();
    response.code = 500;

    // get filename in headers
    let headers = request.headers();
    let filename = headers.get("fileName");
    info!("filename: {:#?}", filename);

    if filename.is_none() {
        return Err(Error::Error("`fileName` not in headers !".to_string()).to_string());
    }

    let mut file_name = String::new();
    if let Some(filename) = filename {
        let name = filename.to_str().map_err(|err| {
            return Error::Error(err.to_string()).to_string();
        })?;
        file_name = name.to_string();
    }

    // file suffix
    let names: Vec<&str> = file_name.split(".").collect();
    let mut file_suffix = String::new();
    if let Some(suffix) = names.last() {
        file_suffix = suffix.to_string()
    }

    let body = request.body();
    if let InvokeBody::Raw(data) = body {
        // image suffix
        if IMAGE_SUFFIXES.contains(&file_suffix.as_str()) {
            // 获取图片的宽度和高度


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
