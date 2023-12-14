//! 文件操作

use crate::analysis::{FileProps, HttpResponse, SuffixProps};
use crate::archive::Archive;
use crate::error::Error;
use crate::preview::Preview;
use crate::utils::file::FileUtils;
use crate::utils::Utils;
use chrono::TimeZone;
use log::info;
use serde_json::{Map, Value};
use std::fs;
use std::fs::Metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::http::HeaderMap;
use tauri::ipc::{InvokeBody, Request};

/// 图片后缀
pub const IMAGE_SUFFIXES: [&str; 11] = ["jpeg", "jpg", "png", "gif", "tiff", "tif", "webp", "ico", "heic", "bmp", "svg"];

pub const OTHER_SUFFIX: [&str; 7] = ["pdf", "xls", "xlsx", "doc", "docx", "ppt", "pptx"];

/// 压缩包后缀
pub const ARCHIVE_SUFFIXES: [&str; 10] = ["zip", "bz2", "gz", "zlib", "tar", "rar", "7z", "tar.xz", "xz", "tgz"];

pub const PREVIEW_FILE: &str = "preview.json";

pub struct Process;

impl Process {
    /// 通过文件流读取文件
    pub fn exec(request: Request) -> Result<HttpResponse, String> {
        let mut response = HttpResponse::default();
        response.code = 500;

        // get filename in headers
        let file_name = Self::get_filename(request.headers())?;

        // file suffix
        let file_suffix = FileUtils::get_file_suffix(&file_name);
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
            return Self::prepare_blob(data, response);
        } else if let InvokeBody::Json(data) = body {
            // json
            return Self::prepare_json(data, response);
        }

        return Ok(response);
    }

    /// 处理二进制数据
    fn prepare_blob(data: &Vec<u8>, mut response: HttpResponse) -> Result<HttpResponse, String> {
        let suffix = response.file_props.suffix.clone();
        let suffix = suffix.as_str();
        // image suffix
        if IMAGE_SUFFIXES.contains(&suffix) {
            // svg 需要单独处理
            let svg = IMAGE_SUFFIXES.get(10).unwrap();
            let content: String;
            if svg == &suffix {
                content = String::from_utf8(data.clone()).map_err(|err| Error::Error(err.to_string()).to_string())?;
            } else {
                content = Utils::generate_image(data.clone());
            }

            response.code = 200;
            response.body = content;
            response.suffix_props = SuffixProps {
                name: suffix.to_string(),
                _type: String::from("image"),
                list: IMAGE_SUFFIXES.iter().map(|str| str.to_string()).collect(),
            };
            info!("success");
            return Ok(response);
        }

        // pdf
        if OTHER_SUFFIX.contains(&suffix) {
            return Preview::exec(response.clone());
        }

        // read content
        let (contents, _, _) = encoding_rs::UTF_8.decode(data);
        response.code = 200;
        response.body = contents.to_string();
        response.suffix_props = SuffixProps {
            name: suffix.to_string(),
            _type: String::new(),
            list: Vec::new(),
        };
        info!("success");
        return Ok(response);
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
        let suffix = &suffix.as_str();
        let file_path = file_path.unwrap().as_str().unwrap();
        info!("file path: {}, suffix: {}", file_path, suffix);

        // 判断文件是否是可执行文件
        info!("prepare to get file `{}` props", file_path);
        let mut file_props = Self::get_file_props(file_path)?;
        if file_props.executable {
            response.error = format!("the `{}` file is an executable file !", file_path);
            return Ok(response);
        }

        file_props.name = response.file_props.name.clone();
        file_props.suffix = response.file_props.suffix.clone();
        file_props.prefix = response.file_props.prefix.clone();
        response.file_props = file_props;

        // cache
        if OTHER_SUFFIX.contains(suffix) || ARCHIVE_SUFFIXES.contains(suffix) {
            return Self::compare_file(suffix, file_path, response);
        }

        Self::prepare_file(suffix, file_path, response)
    }

    /// 读取文件
    fn prepare_file(suffix: &&str, file_path: &str, response: HttpResponse) -> Result<HttpResponse, String> {
        // archive suffix
        if ARCHIVE_SUFFIXES.contains(suffix) {
            let reader = FileUtils::read_file_buffer(file_path)?;
            return Archive::exec(reader, response);
        }

        let content = FileUtils::read_file(file_path)?;
        Self::prepare_blob(&content, response)
    }

    /// 比较临时文件是不是和文件一致
    fn compare_file(suffix: &&str, file_path: &str, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare to get cache ...");
        let name = &response.file_props.name;
        let temp_dir = FileUtils::create_temp_dir(&response.file_props.prefix, false)?;

        let temp_file_path = temp_dir.join(name);
        let temp_file_str = temp_file_path.as_path().to_string_lossy().to_string();
        if temp_file_path.is_dir() || !temp_file_path.exists() {
            info!("path not exists, no cache found ...");
            return Self::prepare_file(&suffix, file_path, response);
        }

        info!("convert `{}` hash ...", &response.file_props.name);
        let hash = FileUtils::get_file_hash(file_path)?;
        let temp_hash = FileUtils::get_file_hash(&temp_file_str)?;
        if hash.is_empty() || temp_hash.is_empty() {
            info!("hash is empty, no cache found ...");
            return Self::prepare_file(&suffix, file_path, response);
        }

        if hash != temp_hash {
            info!("`{}` hash is different ...", &response.file_props.name);
            return Self::prepare_file(&suffix, file_path, response);
        }

        info!("`{}` hash is same ...", &response.file_props.name);

        // 相等则直接读取原来文件
        let json_file_path = temp_dir.clone().join(PREVIEW_FILE);
        let json_file_str = json_file_path.as_path().to_string_lossy().to_string();
        if !json_file_path.exists() {
            info!("json path not exists, no cache found ...");
            return Self::prepare_file(&suffix, file_path, response);
        }

        info!("read `{}` in path `{}`", PREVIEW_FILE, &json_file_str);
        let content = FileUtils::read_file_string(&json_file_str)?;
        if content.is_empty() {
            info!("`{}` content is empty !", PREVIEW_FILE);
            return Self::prepare_file(&suffix, file_path, response);
        }

        info!("convert `{}` to response !", PREVIEW_FILE);
        let response = serde_json::from_str(&content).map_err(|err| Error::Error(err.to_string()).to_string())?;
        info!("get file `{}` by cache success !", &file_path);
        Ok(response)
    }

    /// 从 `headers` 头中获取文件名, 中文名是 encode 的, 需要 decode
    fn get_filename(headers: &HeaderMap) -> Result<String, String> {
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
        return Ok(file_name);
    }

    /// 获取文件属性
    pub(crate) fn get_file_props(file_path: &str) -> Result<FileProps, String> {
        let metadata: Metadata = fs::metadata(file_path).map_err(|err| Error::Error(err.to_string()).to_string())?;
        let mut file_props = FileProps::default();
        file_props.path = file_path.to_string();

        // 获取文件大小
        let size = FileUtils::convert_size(metadata.size());
        file_props.size = size;
        file_props.old_size = metadata.size();

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
        file_props.permissions = FileUtils::format_permissions(mode);

        // 判断文件是不是可以执行(此处有问题?)
        /*
         0o111 的二进制表示是 0b011100100，其中：
           最后一位 001 表示其他用户的执行权限。
           中间一位 100 表示文件所属组的执行权限。
           最高一位 011 表示文件所有者的执行权限。
           如果一个文件的权限位设置为 0o111，那么文件的所有者、所属组和其他用户都具有执行权限，即可以运行该文件
        */
        // file_props.executable = mode & 0o111 != 0;
        file_props.executable = false;
        Ok(file_props)
    }

    /// 读取文件夹下的所有文件
    pub fn read_files(path: &Path, unzip_path_str: &str, size: &mut u64, files: &mut Vec<FileProps>) -> Result<(), String> {
        let entries = fs::read_dir(path).map_err(|err| Error::Error(err.to_string()).to_string())?;
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let filename = entry.file_name().to_str().unwrap_or("").to_string();
            let file_props = Self::get_file_props(&path_str)?;
            let relative_path = path_str.replace(&unzip_path_str, "");

            // suffix
            let suffix = if path.is_dir() {
                String::new()
            } else {
                FileUtils::get_file_suffix(&filename).to_uppercase()
            };

            files.push(FileProps {
                key: path_str.clone(),
                name: suffix.clone(),
                suffix: suffix.clone(),
                prefix: "".to_string(),
                path: relative_path.clone(),
                full_path: path_str.clone(),
                size: if path.is_dir() { String::new() } else { file_props.size },
                old_size: if path.is_dir() { 0 } else { file_props.old_size },
                packed: "".to_string(),
                modified: file_props.modified,
                permissions: "".to_string(),
                executable: file_props.executable,
                kind: suffix.clone(),
                is_directory: path.is_dir(),
                files: vec![],
            });

            if path.is_dir() {
                Self::read_files(&path.clone(), &unzip_path_str, size, files)?;
            } else {
                *size += file_props.old_size;
            }
        }

        Ok(())
    }

    /// 拷贝文件到临时目录, 并把结果写入到文件
    pub fn copy_write_to_file(temp_dir: &PathBuf, response: &HttpResponse) -> Result<(), String> {
        info!("copy origin file to temp dir ...");
        let path = &response.file_props.path;
        let temp_dir_str = temp_dir.as_path().to_string_lossy().to_string();
        let mut copy_options = fs_extra::dir::CopyOptions::new();
        copy_options.overwrite = true;
        fs_extra::copy_items(&[path], &temp_dir_str, &copy_options).map_err(|err| Error::Error(err.to_string()).to_string())?;

        info!("write response into json ...");
        let json_file_path = temp_dir.join(PREVIEW_FILE);
        let file_path = json_file_path.as_path().to_string_lossy().to_string();
        let content = serde_json::to_string_pretty(&response).unwrap(); // 序列化为漂亮格式的 JSON 字符串
        FileUtils::write_to_file_when_clear(&file_path, &content)?;
        Ok(())
    }
}
