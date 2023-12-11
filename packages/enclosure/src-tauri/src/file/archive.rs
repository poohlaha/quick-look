//! 压缩包处理

use std::{fs};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use log::info;
use crate::analysis::{FileProps, HttpResponse, SuffixProps};
use crate::error::Error;
use crate::file::{ARCHIVE_SUFFIXES, FileHandler};

pub struct Archive;

impl Archive {

    /// 获取解压目录
    fn get_program_dir() -> PathBuf {
        let path;
        let data_dir = dirs::data_dir();
        if data_dir.is_none() {
            path = dirs::home_dir().unwrap();
        } else {
            path = data_dir.unwrap()
        }

        return path.join(Path::new("QuickLook"));
    }

    pub fn exec(reader: BufReader<File>, response: HttpResponse) -> Result<HttpResponse, String> {
        let suffix = response.file_props.suffix.clone();

        // zip
        let zip = ARCHIVE_SUFFIXES.get(0).unwrap();
        let bz2 = ARCHIVE_SUFFIXES.get(1).unwrap();

        // flate2 结合 tar
        let gz = ARCHIVE_SUFFIXES.get(2).unwrap();
        let zlib = ARCHIVE_SUFFIXES.get(3).unwrap();
        let tar = ARCHIVE_SUFFIXES.get(4).unwrap();

        // 获取路径(数据目录或home)
        let path = Self::get_program_dir();
        info!("uncompress path: {:#?}", path);

        if &suffix == zip || &suffix == bz2 {
            return Self::prepare_zip(reader, &path, response);
        } else if &suffix == gz || &suffix == zlib || &suffix == tar {
            return Self::prepare_tar(reader, &path, response);
        }

        return Ok(response);
    }

    /// 解压
    fn decompress<F>(kind: String, reader: BufReader<File>, exec_path: &PathBuf, mut response: HttpResponse, func: F) -> Result<HttpResponse, String>
    where
        F: FnOnce(BufReader<File>, &PathBuf, HttpResponse) -> Result<(), String>
    {
        // 解压并放到可执行文件目录
        let name = &response.file_props.name;
        let unzip_dir = exec_path.join(Path::new(name));

        // 如果存在, 则删除目录
        if unzip_dir.exists() {
            fs::remove_dir_all(unzip_dir.clone()).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;
        }

        func(reader, &unzip_dir, response.clone())?;

        // 读取目录下的所有文件
        let mut files: Vec<FileProps> = Vec::new();
        let mut size: u64 = 0;

        let unzip_dir_str = unzip_dir.as_path().to_string_lossy().to_string();
        FileHandler::read_files(unzip_dir.as_path(), &unzip_dir_str, &mut size, &mut files)?;

        // 按目录归纳文件
        let props = Self::organize_files(files);

        let mut files = props.files.clone();
        if files.len() > 0 {
            // 判断第一个名称是不是项目名称, 如果是, 则忽略掉
            let first_file = files.get(0).unwrap();
            let prefix = response.file_props.prefix.clone();
            let spec = String::from("/");
            let mut before_prefix = spec.clone();
            before_prefix.push_str(prefix.as_str());

            let mut after_prefix = prefix.clone();
            after_prefix.push_str(spec.as_str());

            if first_file.name == prefix ||
                first_file.name == before_prefix || first_file.name == after_prefix || first_file.name == spec  {
                files = first_file.files.clone();
            }
        }

        response.code = 200;
        response.file_props.kind = kind;
        response.file_props.packed = FileHandler::convert_size(size);
        response.file_props.size = response.file_props.size;
        response.file_props.files = files;
        response.suffix_props = SuffixProps {
            name: response.file_props.suffix.clone(),
            _type: String::from("archive"),
            list: ARCHIVE_SUFFIXES.iter().map(|str| str.to_string()).collect()
        };

        Ok(response.clone())
    }

    /// zip、bz2
    pub fn prepare_zip(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        let res = Self::decompress("Zip Archive".to_string(), reader, exec_path, response, |reader, exec_path, _| {
            let mut archive = zip::ZipArchive::new(reader).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;

            archive.extract(exec_path).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;

            Ok(())
        })?;

        info!("success");
        Ok(res)
    }

    /// gz、zlib、tar
    pub fn prepare_tar(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        let res = Self::decompress("Tar Archive".to_string(), reader, exec_path, response, |reader, exec_path, _| {
            let gz_decoder = GzDecoder::new(reader);
            let mut file_archive = tar::Archive::new(gz_decoder);
            file_archive.unpack(exec_path).map_err(|err| {
                return Error::Error(err.to_string()).to_string();
            })?;
            Ok(())
        })?;

        info!("success");
        Ok(res)
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
                    new_dir.suffix = if props.is_directory {String::new()} else {FileHandler::get_file_suffix(&name)};
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