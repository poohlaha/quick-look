//! 文件操作

mod archive;

use std::fs;
use std::fs::{File, Metadata};
use std::io::{BufReader, Read};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::time::SystemTime;
use crate::error::Error;
use base64::Engine;
use chrono::TimeZone;
use log::info;
use serde_json::{Map, Value};
use tauri::http::HeaderMap;
use tauri::ipc::{InvokeBody, Request};
use crate::analysis::{FileProps, HttpResponse, SuffixProps};
use crate::file::archive::Archive;


/// 图片后缀
const IMAGE_SUFFIXES: [&str; 11] = ["jpeg", "jpg", "png", "gif", "tiff", "tif", "webp", "ico", "heic", "svg", "bmp"];

/// 压缩包后缀
const ARCHIVE_SUFFIXES: [&str; 7] = ["zip", "bz2", "gz", "tar", "rar", "7z",  "xz"];


pub struct FileHandler;

impl FileHandler {

    /// 通过文件流读取文件
    pub fn exec(request: Request) -> Result<HttpResponse, String> {
        let mut response = HttpResponse::default();
        response.code = 500;

        // get filename in headers
        let file_name = Self::get_filename(request.headers())?;

        // file suffix
        let file_suffix = Self::get_file_suffix(&file_name);
        response.file_props.name = file_name.clone();
        response.file_props.suffix = file_suffix.clone();

        if let Some(index) = file_name.rfind('.') {
            let name = &file_name[..index];
            response.file_props.prefix = name.to_string();
        } else {
            response.file_props.prefix = file_name.clone();
        }

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
        let suffix = response.file_props.suffix.clone();
        if IMAGE_SUFFIXES.contains(&suffix.as_str()) {
            // image suffix
            // 转成 base64
            let str = base64::engine::general_purpose::STANDARD.encode::<Vec<u8>>(data.clone());
            let mut content = String::from("data:image/png;base64,");
            content.push_str(&str);

            response.code = 200;
            response.body = content;
            response.suffix_props = SuffixProps {
                name: suffix.clone(),
                _type: String::from("image"),
                list: IMAGE_SUFFIXES.iter().map(|str| str.to_string()).collect()
            };
            info!("success");
            return Ok(response);
        } else {
            let content = String::from_utf8(data.to_vec()).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;

            response.code = 200;
            response.body = content;
            response.suffix_props = SuffixProps {
                name: suffix.clone(),
                _type: String::new(),
                list: Vec::new()
            };
            info!("success");
            return Ok(response);
        }
    }

    /// 读取 json 数据
    fn prepare_json(data: &Value, mut response: HttpResponse) -> Result<HttpResponse, String> {
        let map = Map::new();
        let obj: &Map<String, Value> = data.as_object().unwrap_or(&map);
        let file_path = obj.get("filePath");
        if file_path.is_none() {
            return Err(Error::Error("`fileName` not in headers !".to_string()).to_string());
        }

        let suffix = response.file_props.suffix.clone();
        let file_path = file_path.unwrap().as_str().unwrap();
        info!("file path: {}", file_path);

        // 判断文件是否是可执行文件
       let mut file_props = Self::get_file_props(file_path)?;
        if file_props.executable {
            response.error = format!("the `{}` file is an executable file !", file_path);
            return Ok(response);
        }

        file_props.name = response.file_props.name.clone();
        file_props.suffix = response.file_props.suffix.clone();
        response.file_props = file_props;

        // archive suffix
        if ARCHIVE_SUFFIXES.contains(&suffix.as_str()) {
            let reader = Self::read_file_buffer(file_path)?;
            return Archive::exec(reader, response)
        }

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

    /// 读取文件流
    fn read_file_buffer(file_path: &str) -> Result<BufReader<File>, String> {
        let file = File::open(&file_path).map_err(|err| {
            return Error::Error(err.to_string()).to_string();
        })?;

        Ok(BufReader::new(file))
    }

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

    /// 获取文件属性
    pub(crate) fn get_file_props(file_path: &str) -> Result<FileProps, String> {
        let metadata: Metadata = fs::metadata(file_path).map_err(|err| {
            return Error::Error(err.to_string()).to_string();
        })?;

        let mut file_props = FileProps::default();
        file_props.path = file_path.to_string();

        // 获取文件大小
        let size = Self::convert_size(metadata.size());
        file_props.size = size;

        // 获取文件或目录的最后修改时间
        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        // 获取毫秒级的时间戳
        let milliseconds = modified.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as i64;

        // 指定时区为 UTC
        let utc = chrono::Utc.timestamp_millis_opt(milliseconds).unwrap();

        // 获取正确时区的时间
        let local_time = utc.with_timezone(&chrono::Local);
        file_props.modified = local_time.format("%Y/%m/%d %H:%M").to_string();

        // 获取文件或目录的权限信息
        let mode = metadata.permissions().mode();
        file_props.permissions = Self::format_permissions(mode);

        // 判断文件是不是可以执行
        /*
          0o111 的二进制表示是 0b011100100，其中：
            最后一位 001 表示其他用户的执行权限。
            中间一位 100 表示文件所属组的执行权限。
            最高一位 011 表示文件所有者的执行权限。
            如果一个文件的权限位设置为 0o111，那么文件的所有者、所属组和其他用户都具有执行权限，即可以运行该文件
         */
        file_props.executable = mode & 0o111 != 0;
        Ok(file_props)
    }

    fn format_permissions(mode: u32) -> String {
        let user = Self::format_mode_part((mode >> 6) & 0o7);
        let group = Self::format_mode_part((mode >> 3) & 0o7);
        let others = Self::format_mode_part(mode & 0o7);
        format!("{}{}{}", user, group, others)
    }

    fn format_mode_part(part: u32) -> String {
        let r = if (part & 0o4) == 0 { "-" } else { "r" };
        let w = if (part & 0o2) == 0 { "-" } else { "w" };
        let x = if (part & 0o1) == 0 { "-" } else { "x" };
        format!("{}{}{}", r, w, x)
    }

    /// 转换文件大小
    pub(crate) fn convert_size(size: u64) -> String {
        if size >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if size >= 1024 * 1024 {
            format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
        } else if size >= 1024 {
            format!("{:.2} KB", size as f64 / 1024.0)
        } else {
            format!("{} bytes", size)
        }
    }
}