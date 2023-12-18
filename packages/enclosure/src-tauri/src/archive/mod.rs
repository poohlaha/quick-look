//! 压缩包处理

use crate::analysis::{HttpResponse, SuffixProps};
use crate::error::Error;
use crate::process::{Process, ARCHIVE_SUFFIXES};
use crate::utils::file::FileUtils;
use bzip2::read::{BzDecoder, BzEncoder};
use bzip2::Compression;
use flate2::read::GzDecoder;
use log::info;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use xz2::read::XzEncoder;

pub struct Archive;

impl Archive {
    pub fn exec(reader: BufReader<File>, mut response: HttpResponse) -> Result<HttpResponse, String> {
        let suffix = response.file_props.suffix.clone();

        // zip
        let zip = ARCHIVE_SUFFIXES.get(0).unwrap();

        // bz2
        let bz2 = ARCHIVE_SUFFIXES.get(1).unwrap();

        // flate2 结合 tar
        let gz = ARCHIVE_SUFFIXES.get(2).unwrap();
        let zlib = ARCHIVE_SUFFIXES.get(3).unwrap();
        let tar = ARCHIVE_SUFFIXES.get(4).unwrap();

        // rar
        let rar = ARCHIVE_SUFFIXES.get(5).unwrap();

        // 7z
        let z7 = ARCHIVE_SUFFIXES.get(6).unwrap();

        // tar.xz
        let tar_xz = ARCHIVE_SUFFIXES.get(7).unwrap();

        // xz
        let xz = ARCHIVE_SUFFIXES.get(8).unwrap();

        // tgz
        let tgz = ARCHIVE_SUFFIXES.get(9).unwrap();

        let name = &response.file_props.name;
        let temp_path = FileUtils::create_temp_dir(&response.file_props.prefix, true)?;
        if &suffix == zip {
            return Self::prepare_zip(reader, &temp_path, response);
        }

        if &suffix == bz2 {
            return Self::prepare_bz2(reader, &temp_path, response);
        }

        if &suffix == gz || &suffix == zlib || &suffix == tar || &suffix == tgz {
            return Self::prepare_tar(reader, &temp_path, response);
        }

        if &suffix == rar {
            return Self::prepare_rar(reader, &temp_path, response);
        }

        if name.ends_with(tar_xz) {
            return Self::prepare_tar_xz(reader, &temp_path, response);
        }

        if &suffix == xz {
            return Self::prepare_xz(reader, &temp_path, response);
        }

        if &suffix == z7 {
            return Self::prepare_7z(reader, &temp_path, response);
        }

        response.error = "读取压缩包失败, 不支持的格式".to_string();
        return Ok(response);
    }

    /// 解压
    fn decompress<F>(kind: String, reader: BufReader<File>, unzip_path: &PathBuf, mut response: HttpResponse, func: F) -> Result<HttpResponse, String>
    where
        F: FnOnce(BufReader<File>, &PathBuf, HttpResponse) -> Result<(), String>,
    {
        func(reader, &unzip_path, response.clone())?;

        // 读取目录下的所有文件,并归纳目录
        let (files, size) = Process::read_directory(unzip_path, &response.file_props.prefix)?;

        response.code = 200;
        response.file_props.kind = kind;
        response.file_props.packed = response.file_props.size;
        response.file_props.size = FileUtils::convert_size(size);
        response.file_props.files = files;
        response.file_props.full_path = unzip_path.as_path().to_string_lossy().to_string();
        response.suffix_props = SuffixProps {
            name: response.file_props.suffix.clone(),
            _type: String::from("archive"),
            list: ARCHIVE_SUFFIXES.iter().map(|str| str.to_string()).collect(),
        };

        // 拷贝数据, 写入文件
        // 写入到 json 文件
        Process::copy_write_to_file(&unzip_path, &response)?;
        Ok(response.clone())
    }

    /// zip
    pub fn prepare_zip(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare zip ...");

        let res = Self::decompress("ZIP Archive".to_string(), reader, exec_path, response, |reader, unzip_path, _| {
            let mut archive = zip::ZipArchive::new(reader).map_err(|err| Error::Error(err.to_string()).to_string())?;
            archive.extract(unzip_path).map_err(|err| Error::Error(err.to_string()).to_string())?;
            Ok(())
        })?;

        info!("prepare zip success !");
        Ok(res)
    }

    /// bz2
    pub fn prepare_bz2(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare bz2 ...");
        let res = Self::decompress("BZ2 Archive".to_string(), reader, exec_path, response, |reader, unzip_path, response| {
            let compressor = BzEncoder::new(reader, Compression::best());
            let mut decompressor = BzDecoder::new(compressor);

            // 读取解压缩的数据
            let mut buffer = Vec::new();
            decompressor
                .read_to_end(&mut buffer)
                .map_err(|err| Error::Error(err.to_string()).to_string())?;

            let bz2 = ARCHIVE_SUFFIXES.get(1).unwrap();
            let mut name = response.file_props.name.clone();
            name = name.replace(&format!(".{}", bz2), "");

            let file_path = unzip_path.join(&name);
            let mut output_file = File::create(file_path).map_err(|err| Error::Error(err.to_string()).to_string())?;
            output_file.write_all(&buffer).map_err(|err| Error::Error(err.to_string()).to_string())?;
            Ok(())
        })?;

        info!("prepare bz2 success !");
        Ok(res)
    }

    /// gz、zlib、tar
    pub fn prepare_tar(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare tar ...");

        let res = Self::decompress("TAR Archive".to_string(), reader, exec_path, response, |reader, unzip_path, _| {
            let gz_decoder = GzDecoder::new(reader);
            let mut file_archive = tar::Archive::new(gz_decoder);
            file_archive.unpack(unzip_path).map_err(|err| Error::Error(err.to_string()).to_string())?;
            Ok(())
        })?;

        info!("prepare tar success!");
        Ok(res)
    }

    /// rar: `rar a xxx.rar .`
    pub fn prepare_rar(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare rar ...");

        let res = Self::decompress("Rar Archive".to_string(), reader, exec_path, response, |_, unzip_path, response| {
            let file_path = response.file_props.path.clone();
            let mut archive = unrar::Archive::new(file_path)
                .extract_to(unzip_path.to_string_lossy().to_string())
                .map_err(|err| Error::Error(err.to_string()).to_string())?;
            archive.process().map_err(|err| Error::Error(err.to_string()).to_string())?;
            Ok(())
        })?;

        info!("prepare rar success !");
        Ok(res)
    }

    /// tar.xz
    pub fn prepare_tar_xz(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare tar.xz ...");
        let res = Self::decompress("XZ Archive".to_string(), reader, exec_path, response, |reader, _, _| {
            let decoder = XzEncoder::new(reader, 9);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(exec_path).map_err(|err| Error::Error(err.to_string()).to_string())?;
            Ok(())
        })?;

        info!("prepare tar.xz success !");
        Ok(res)
    }

    /// xz
    pub fn prepare_xz(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare xz ...");
        let res = Self::decompress("XZ Archive".to_string(), reader, exec_path, response, |reader, unzip_path, response| {
            let name = &response.file_props.prefix;
            let file_path = unzip_path.join(name);

            let mut output_file = File::create(&file_path).map_err(|err| Error::Error(err.to_string()).to_string())?;
            let mut decoder = XzEncoder::new(reader, 9);
            io::copy(&mut decoder, &mut output_file).map_err(|err| Error::Error(err.to_string()).to_string())?;
            Ok(())
        })?;

        info!("prepare xz success !");
        Ok(res)
    }

    /// 7z
    pub fn prepare_7z(reader: BufReader<File>, exec_path: &PathBuf, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare 7z ...");
        let res = Self::decompress("7Z Archive".to_string(), reader, exec_path, response, |_, unzip_path, response| {
            sevenz_rust::decompress_file(&Path::new(&response.file_props.path), unzip_path)
                .map_err(|err| Error::Error(err.to_string()).to_string())?;
            Ok(())
        })?;

        info!("prepare 7z success !");
        Ok(res)
    }
}
