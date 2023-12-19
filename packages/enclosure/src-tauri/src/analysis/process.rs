//! 处理文件

use crate::analysis::archive::Archive;
use crate::analysis::document::Document;
use crate::cache::Cache;
use crate::config::FileProps;
use crate::config::{HttpResponse, SuffixProps, ARCHIVE_SUFFIXES, DOCUMENT_SUFFIX, IMAGE_SUFFIXES, PREVIEW_FILE};
use crate::error::Error;
use crate::prepare::{Prepare, Treat};
use crate::utils::file::FileUtils;
use crate::utils::Utils;
use chrono::TimeZone;
use log::{error, info};
use std::collections::HashMap;
use std::fs;
use std::fs::Metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::ipc::{InvokeBody, Request};
use tauri::AppHandle;

pub struct Process;

impl Treat<HttpResponse> for Process {
    /// 处理
    fn handle(app: &AppHandle, request: Request) -> Result<HttpResponse, String> {
        let file_name = Self::get_filename(request.headers())?; // get filename in headers
        let response = Self::get_response(&file_name);
        Self::prepare(app, request.body(), response)
    }

    /// 从 `headers` 头中获取文件名, 中文名是 encode 的, 需要 decode
    fn get_filename(headers: &tauri::http::HeaderMap) -> Result<String, String> {
        info!("request headers: {:#?}", headers);
        let filename = headers.get("fileName");
        info!("filename in header: {:#?}", filename);
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
        info!("filename: {:#?}", &file_name);
        return Ok(file_name);
    }

    /// response
    fn get_response(filename: &str) -> HttpResponse {
        let mut response = HttpResponse::default();
        response.code = 500;

        // file suffix
        let file_suffix = FileUtils::get_file_suffix(&filename);
        response.file_props.name = filename.to_string();
        response.file_props.suffix = file_suffix.clone();

        if let Some(index) = filename.rfind('.') {
            let name = &filename[..index];
            response.file_props.prefix = name.to_string();
        } else {
            response.file_props.prefix = filename.to_string();
        }

        response
    }

    fn prepare(_: &AppHandle, body: &InvokeBody, response: HttpResponse) -> Result<HttpResponse, String> {
        // blob
        if let InvokeBody::Raw(data) = body {
            let res = Self::prepare_blob(data, response.clone())?;
            return Ok(res);
        }

        // json
        if let InvokeBody::Json(data) = body {
            let params = Self::get_params_by_header(data);
            let mut file_type = String::new();
            if let Some(param_type) = params.get("fileType") {
                file_type = param_type.to_string();
            }

            return if let Some(param_path) = params.get("filePath") {
                if param_path.is_empty() {
                    return Err(Error::Error("`fileName` not in headers !".to_string()).to_string());
                }

                let res = Self::prepare_json(param_path, response.clone())?;
                if file_type.is_empty() {
                    Cache::save_history(&res.file_props)?;

                    // 更新菜单 some errors ?
                    // Menu::update_history_submenus(app);
                }

                Ok(res)
            } else {
                Err(Error::Error("`fileName` not in headers !".to_string()).to_string())
            };
        }

        return Ok(response);
    }

    fn prepare_blob(data: &Vec<u8>, response: HttpResponse) -> Result<HttpResponse, String> {
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

            let res = Self::prepare_response(response, Some(content), "image");
            info!("get image data success!");
            return Ok(res);
        }

        // pdf, doc
        if DOCUMENT_SUFFIX.contains(&suffix) {
            return Document::with_response(response.clone());
        }

        // read content
        let (contents, _, _) = encoding_rs::UTF_8.decode(data);
        let res = Self::prepare_response(response, Some(contents.to_string()), "");
        info!("get blob data success");
        return Ok(res);
    }

    fn prepare_json(file_path: &str, response: HttpResponse) -> Result<HttpResponse, String> {
        let mut res = response.clone();
        let suffix = res.file_props.suffix.clone();
        let suffix = &suffix.as_str();
        info!("file path: {}, suffix: {}", file_path, suffix);

        // 判断文件是否是可执行文件
        info!("prepare to get file `{}` props", file_path);
        let mut file_props = Self::prepare_file_props(file_path)?;
        if file_props.executable {
            res.error = format!("the `{}` file is an executable file !", file_path);
            return Ok(res);
        }

        file_props.name = res.file_props.name.clone();
        file_props.suffix = res.file_props.suffix.clone();
        file_props.prefix = res.file_props.prefix.clone();
        res.file_props = file_props;

        // directory
        let path = PathBuf::from(file_path);
        if path.is_dir() {
            return Self::prepare_directory(&path, res);
        }

        // cache
        if DOCUMENT_SUFFIX.contains(suffix) || ARCHIVE_SUFFIXES.contains(suffix) {
            return Self::compare_file(file_path, res);
        }

        Self::prepare_file(file_path, res)
    }

    fn prepare_directory(path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        let file_path = path.as_path().to_string_lossy().to_string();
        if !path.exists() {
            error!("file path `{}` not exists, read directory error !", &file_path);
            return Err(Error::Error(format!("file path `{}` not exists, read directory error !", &file_path)).to_string());
        }

        // 按目录归纳文件
        let (files, size) = Self::read_directory(path, &response.file_props.prefix)?;

        let mut res = response.clone();
        res.file_props.kind = "Directory".to_string();
        res.file_props.size = FileUtils::convert_size(size);
        res.file_props.files = files;
        res.file_props.full_path = path.as_path().to_string_lossy().to_string();
        let res = Self::prepare_response(response, None, "dir");
        Ok(res)
    }

    fn prepare_file(file_path: &str, response: HttpResponse) -> Result<HttpResponse, String> {
        let suffix = &response.file_props.suffix;
        // archive
        if ARCHIVE_SUFFIXES.contains(&suffix.as_str()) {
            let reader = FileUtils::read_file_buffer(file_path)?;
            return Archive::with_file_reader(reader, response);
        }

        let content = FileUtils::read_file(file_path)?;
        Self::prepare_blob(&content, response)
    }

    /// 比较临时文件是不是和文件一致
    fn compare_file(file_path: &str, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare to get cache ...");
        let name = &response.file_props.name;
        let temp_dir = FileUtils::create_temp_dir(&response.file_props.prefix, false)?;

        let temp_file_path = temp_dir.join(name);
        let temp_file_str = temp_file_path.as_path().to_string_lossy().to_string();
        if temp_file_path.is_dir() || !temp_file_path.exists() {
            info!("path not exists, no cache found ...");
            return Self::prepare_file(file_path, response);
        }

        info!("convert `{}` hash ...", &response.file_props.name);
        let hash = FileUtils::get_file_hash(file_path)?;
        let temp_hash = FileUtils::get_file_hash(&temp_file_str)?;
        if hash.is_empty() || temp_hash.is_empty() {
            info!("hash is empty, no cache found ...");
            return Self::prepare_file(file_path, response);
        }

        if hash != temp_hash {
            info!("`{}` hash is different ...", &response.file_props.name);
            return Self::prepare_file(file_path, response);
        }

        info!("`{}` hash is same ...", &response.file_props.name);

        // 相等则直接读取原来文件
        let json_file_path = temp_dir.clone().join(PREVIEW_FILE);
        let json_file_str = json_file_path.as_path().to_string_lossy().to_string();
        if !json_file_path.exists() {
            info!("json path not exists, no cache found ...");
            return Self::prepare_file(file_path, response);
        }

        info!("read `{}` in path `{}`", PREVIEW_FILE, &json_file_str);
        let content = FileUtils::read_file_string(&json_file_str)?;
        if content.is_empty() {
            info!("`{}` content is empty !", PREVIEW_FILE);
            return Self::prepare_file(file_path, response);
        }

        info!("convert `{}` to response !", PREVIEW_FILE);
        let response = serde_json::from_str(&content).map_err(|err| Error::Error(err.to_string()).to_string())?;
        info!("get file `{}` by cache success !", &file_path);
        Ok(response)
    }

    /// 获取文件属性
    fn prepare_file_props(file_path: &str) -> Result<FileProps, String> {
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

    /// 获取输出结果
    fn prepare_response(response: HttpResponse, content: Option<String>, _type: &str) -> HttpResponse {
        let mut res = response.clone();
        res.code = 200;

        // content
        if let Some(content) = content {
            res.body = content;
        }

        // get suffix list
        let mut list: Vec<String> = Vec::new();
        if _type == "image" {
            list = IMAGE_SUFFIXES.iter().map(|str| str.to_string()).collect();
        }

        res.suffix_props = SuffixProps {
            name: response.file_props.suffix.clone(),
            _type: String::from(_type),
            list,
        };

        res
    }
}

impl Process {
    /// 根据路径执行
    pub fn exec_by_file_path(filename: &str, file_path: &str) -> Result<HttpResponse, String> {
        let response = Self::get_response(&filename);
        Self::prepare_json(file_path, response)
    }

    fn get_params_by_header(data: &serde_json::Value) -> HashMap<String, String> {
        let map = serde_json::Map::new();
        let obj: &serde_json::Map<String, serde_json::Value> = data.as_object().unwrap_or(&map);
        let file_type = obj.get("fileType");
        let file_path = obj.get("filePath");

        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(file_type) = file_type {
            params.insert(String::from("fileType"), file_type.as_str().unwrap().to_string());
        }

        if let Some(file_path) = file_path {
            params.insert(String::from("filePath"), file_path.as_str().unwrap().to_string());
        }

        params
    }

    /// 读取目录
    pub fn read_directory(path: &PathBuf, prefix: &str) -> Result<(Vec<FileProps>, u64), String> {
        // 读取目录下的所有文件
        let mut files: Vec<FileProps> = Vec::new();
        let mut size: u64 = 0;

        let path_str = path.as_path().to_string_lossy().to_string();
        Process::read_files(path.as_path(), &path_str, &mut size, &mut files)?;

        // 按目录归纳文件
        let props = Self::organize_files(files);
        let mut files = props.files.clone();
        if files.len() > 0 {
            // 判断第一个名称是不是项目名称, 如果是, 则忽略掉
            let first_file = files.get(0).unwrap();
            let spec = String::from("/");
            let mut before_prefix = spec.clone();
            before_prefix.push_str(prefix);

            let mut after_prefix = prefix.to_string();
            after_prefix.push_str(spec.as_str());

            if first_file.name == prefix || first_file.name == before_prefix || first_file.name == after_prefix || first_file.name == spec {
                files = first_file.files.clone();
            }
        }

        Ok((files, size))
    }

    /// 读取文件夹下的所有文件
    pub fn read_files(path: &Path, unzip_path_str: &str, size: &mut u64, files: &mut Vec<FileProps>) -> Result<(), String> {
        let entries = fs::read_dir(path).map_err(|err| Error::Error(err.to_string()).to_string())?;
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let filename = entry.file_name().to_str().unwrap_or("").to_string();
            let file_props = Self::prepare_file_props(&path_str)?;
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

    /// 按目录归纳文件
    fn organize_files(files: Vec<FileProps>) -> FileProps {
        let mut root = FileProps::default();

        for props in files {
            let file_path = &props.path;

            let path = Path::new(file_path);
            let mut current_dir = &mut root;
            let mut full_path = PathBuf::new();

            for component in path.iter() {
                let name = component.to_string_lossy().to_string();
                let index = current_dir.files.iter().position(|d| d.name == name);
                full_path = full_path.join(&name);

                if let Some(index) = index {
                    // 目录已存在，继续向下
                    current_dir = &mut current_dir.files[index];
                } else {
                    // 目录不存在，添加新目录
                    let mut new_dir = props.clone();
                    new_dir.name = name.clone();
                    new_dir.path = full_path.clone().as_path().to_string_lossy().to_string();
                    new_dir.files = Vec::new();
                    new_dir.suffix = if props.is_directory {
                        String::new()
                    } else {
                        FileUtils::get_file_suffix(&name)
                    };
                    new_dir.kind = FileUtils::get_file_suffix(&name);

                    current_dir.files.push(new_dir);

                    // 更新 current_dir 的引用
                    let len = current_dir.files.len();
                    current_dir = &mut current_dir.files[len - 1];
                }
            }
        }

        root
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
