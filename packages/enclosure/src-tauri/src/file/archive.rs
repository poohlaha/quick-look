//! 压缩包处理

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use log::info;
use crate::analysis::{FileProps, HttpResponse, SuffixProps};
use crate::error::Error;
use crate::file::{ARCHIVE_SUFFIXES, FileHandler};

pub struct Archive;

impl Archive {

    pub fn exec(reader: BufReader<File>, response: HttpResponse) -> Result<HttpResponse, String> {
        let suffix = response.file_props.suffix.clone();

        let zip = ARCHIVE_SUFFIXES.get(0).unwrap();
        let bz2 = ARCHIVE_SUFFIXES.get(1).unwrap();

        // use zip
        if &suffix == zip || &suffix == bz2 {
            return Self::prepare_zip(reader, response);
        }

        return Ok(response);
    }

    pub fn prepare_zip(reader: BufReader<File>, mut response: HttpResponse) -> Result<HttpResponse, String> {
        let mut archive = zip::ZipArchive::new(reader).map_err(|err| {
            return Error::Error(err.to_string()).to_string();
        })?;

        let mut files: Vec<FileProps> = Vec::new();
        let mut zip_packed = 0;

        for i in 0 .. archive.len() {
            let file = archive.by_index(i).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;


            let out_path: &Path = file.enclosed_name().unwrap();

            // 实际大小
            let size = if file.is_dir() {String::new()} else {FileHandler::convert_size(file.size())};

            // 压缩大小
            let packed = if file.is_dir() {String::new()} else {FileHandler::convert_size(file.compressed_size())};

            zip_packed += file.compressed_size();

            // 最后修改时间
            let modified = file.last_modified();
            let modified = format!("{}/{}/{} {}:{}", modified.year(), modified.month(), modified.day(), modified.hour(), modified.minute());

            let suffix = FileHandler::get_file_suffix(file.name()).to_uppercase();
            files.push(FileProps {
                key: file.name().to_string(),
                name: file.name().to_string(),
                suffix: suffix.clone(),
                prefix: "".to_string(),
                path: out_path.to_string_lossy().to_string(),
                size,
                packed,
                modified,
                permissions: "".to_string(),
                executable: false,
                kind: suffix.clone(),
                is_directory: file.is_dir(),
                files: vec![],
            });
        }

        // 按目录归纳文件
        let props = Self::organize_files(files, &response.file_props.prefix);
        let mut props_files: Vec<FileProps> = Vec::new();
        let files = props.files.clone();
        if files.len() > 0 {
            props_files = files.get(0).unwrap().files.clone();
        }

        response.code = 200;
        response.file_props.kind = "Zip Archive".to_string();
        response.file_props.packed = FileHandler::convert_size(zip_packed);
        response.file_props.files = props_files;
        response.suffix_props = SuffixProps {
            name: response.file_props.suffix.clone(),
            _type: String::from("archive"),
            list: ARCHIVE_SUFFIXES.iter().map(|str| str.to_string()).collect()
        };

        info!("success");
        Ok(response)
    }

    /// 按目录归纳文件
    fn organize_files(files: Vec<FileProps>, parent: &str) -> FileProps {
        let mut root = FileProps::default();

        for props in files {
          let file_path = &props.path;
            // 根目录不需要
            if file_path == parent {
                continue;
            }

            let file_path = file_path.strip_prefix(parent).unwrap_or("");
            if file_path.is_empty() {
                continue;
            }

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
                    new_dir.suffix = FileHandler::get_file_suffix(&name);
                    new_dir.kind = FileHandler::get_file_suffix(&name);

                    current_dir.files.push(new_dir);

                    // 更新 current_dir 的引用
                    let len = current_dir.files.len();
                    current_dir = &mut current_dir.files[len - 1];

                }
            }
        }

        root
    }
}