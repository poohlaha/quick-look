//! 压缩包处理

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
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
                name: file.name().to_string(),
                suffix: suffix.clone(),
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

        response.code = 200;
        response.file_props.kind = "Zip Archive".to_string();
        response.file_props.packed = FileHandler::convert_size(zip_packed);
        response.file_props.files = files;
        response.suffix_props = SuffixProps {
            name: response.file_props.suffix.clone(),
            _type: String::from("archive"),
            list: ARCHIVE_SUFFIXES.iter().map(|str| str.to_string()).collect()
        };

        info!("success");
        Ok(response)
    }
}