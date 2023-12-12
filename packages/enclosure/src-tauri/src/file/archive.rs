//! 压缩包处理

use crate::analysis::{FileProps, HttpResponse, SuffixProps};
use crate::error::Error;
use crate::file::{FileHandler, ARCHIVE_SUFFIXES};
use bzip2::read::{BzDecoder, BzEncoder};
use bzip2::Compression;
use flate2::read::GzDecoder;
use log::{error, info};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use xz2::read::XzEncoder;

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

    // 获取路径(数据目录或home)
    let path = Self::get_program_dir();
    info!("uncompress path: {:#?}", path);

    let name = &response.file_props.name;

    if &suffix == zip {
      return Self::prepare_zip(reader, &path, response);
    }

    if &suffix == bz2 {
      return Self::prepare_bz2(reader, &path, response);
    }

    if &suffix == gz || &suffix == zlib || &suffix == tar {
      return Self::prepare_tar(reader, &path, response);
    }

    if &suffix == rar {
      return Self::prepare_rar(reader, &path, response);
    }

    if name.ends_with(tar_xz) {
      return Self::prepare_tar_xz(reader, &path, response);
    }

    if &suffix == xz {
      return Self::prepare_xz(reader, &path, response);
    }

    if &suffix == z7 {
      return Self::prepare_7z(reader, &path, response);
    }

    response.error = "读取压缩包失败, 不支持的格式".to_string();
    return Ok(response);
  }

  /// 解压
  fn decompress<F>(
    kind: String,
    reader: BufReader<File>,
    exec_path: &PathBuf,
    mut response: HttpResponse,
    func: F,
  ) -> Result<HttpResponse, String>
  where
    F: FnOnce(BufReader<File>, &PathBuf, HttpResponse) -> Result<(), String>,
  {
    // 解压并放到可执行文件目录
    let name = response.file_props.prefix.clone();
    let unzip_path = exec_path.join(Path::new(&name));

    // 如果存在, 则删除目录
    if unzip_path.exists() {
      fs::remove_dir_all(unzip_path.clone()).map_err(|err| {
        let err_msg = Error::Error(err.to_string()).to_string();
        error!("{},{},{}", file!(), line!(), err_msg);
        return err_msg;
      })?;
    }

    fs::create_dir_all(&unzip_path).map_err(|err| {
      let err_msg = Error::Error(err.to_string()).to_string();
      error!("{},{},{}", file!(), line!(), err_msg);
      return err_msg;
    })?;

    func(reader, &unzip_path, response.clone())?;

    // 读取目录下的所有文件
    let mut files: Vec<FileProps> = Vec::new();
    let mut size: u64 = 0;

    let unzip_dir_str = unzip_path.as_path().to_string_lossy().to_string();
    FileHandler::read_files(unzip_path.as_path(), &unzip_dir_str, &mut size, &mut files)?;

    // 按目录归纳文件
    let props = Self::organize_files(files);
    println!("props: {:#?}", props);

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

      if first_file.name == prefix
        || first_file.name == before_prefix
        || first_file.name == after_prefix
        || first_file.name == spec
      {
        files = first_file.files.clone();
      }
    }

    response.code = 200;
    response.file_props.kind = kind;
    response.file_props.packed = response.file_props.size;
    response.file_props.size = FileHandler::convert_size(size);
    response.file_props.files = files;
    response.suffix_props = SuffixProps {
      name: response.file_props.suffix.clone(),
      _type: String::from("archive"),
      list: ARCHIVE_SUFFIXES.iter().map(|str| str.to_string()).collect(),
    };

    Ok(response.clone())
  }

  /// zip
  pub fn prepare_zip(
    reader: BufReader<File>,
    exec_path: &PathBuf,
    response: HttpResponse,
  ) -> Result<HttpResponse, String> {
    let res = Self::decompress(
      "ZIP Archive".to_string(),
      reader,
      exec_path,
      response,
      |reader, unzip_path, _| {
        let mut archive = zip::ZipArchive::new(reader).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        archive.extract(unzip_path).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        Ok(())
      },
    )?;

    info!("success");
    Ok(res)
  }

  /// bz2
  pub fn prepare_bz2(
    reader: BufReader<File>,
    exec_path: &PathBuf,
    response: HttpResponse,
  ) -> Result<HttpResponse, String> {
    let res = Self::decompress(
      "BZ2 Archive".to_string(),
      reader,
      exec_path,
      response,
      |reader, unzip_path, response| {
        let compressor = BzEncoder::new(reader, Compression::best());
        let mut decompressor = BzDecoder::new(compressor);

        // 读取解压缩的数据
        let mut buffer = Vec::new();
        decompressor.read_to_end(&mut buffer).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        let bz2 = ARCHIVE_SUFFIXES.get(1).unwrap();
        let mut name = response.file_props.name.clone();
        name = name.replace(&format!(".{}", bz2), "");

        let file_path = unzip_path.join(&name);

        let mut output_file = File::create(file_path).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        output_file.write_all(&buffer).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        Ok(())
      },
    )?;

    info!("success");
    Ok(res)
  }

  /// gz、zlib、tar
  pub fn prepare_tar(
    reader: BufReader<File>,
    exec_path: &PathBuf,
    response: HttpResponse,
  ) -> Result<HttpResponse, String> {
    let res = Self::decompress(
      "TAR Archive".to_string(),
      reader,
      exec_path,
      response,
      |reader, unzip_path, _| {
        let gz_decoder = GzDecoder::new(reader);
        let mut file_archive = tar::Archive::new(gz_decoder);
        file_archive.unpack(unzip_path).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;
        Ok(())
      },
    )?;

    info!("success");
    Ok(res)
  }

  /// rar: `rar a xxx.rar .`
  pub fn prepare_rar(
    reader: BufReader<File>,
    exec_path: &PathBuf,
    response: HttpResponse,
  ) -> Result<HttpResponse, String> {
    let res = Self::decompress(
      "Rar Archive".to_string(),
      reader,
      exec_path,
      response,
      |_, unzip_path, response| {
        let file_path = response.file_props.path.clone();
        let mut archive = unrar::Archive::new(file_path)
          .extract_to(unzip_path.to_string_lossy().to_string())
          .map_err(|err| {
            let err_msg = Error::Error(err.to_string()).to_string();
            error!("{},{},{}", file!(), line!(), err_msg);
            return err_msg;
          })?;

        archive.process().map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        Ok(())
      },
    )?;

    info!("success");
    Ok(res)
  }

  /// tar.xz
  pub fn prepare_tar_xz(
    reader: BufReader<File>,
    exec_path: &PathBuf,
    response: HttpResponse,
  ) -> Result<HttpResponse, String> {
    let res = Self::decompress(
      "XZ Archive".to_string(),
      reader,
      exec_path,
      response,
      |reader, _, _| {
        let decoder = XzEncoder::new(reader, 9);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(exec_path).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        Ok(())
      },
    )?;

    info!("success");
    Ok(res)
  }

  /// xz
  pub fn prepare_xz(
    reader: BufReader<File>,
    exec_path: &PathBuf,
    response: HttpResponse,
  ) -> Result<HttpResponse, String> {
    let res = Self::decompress(
      "XZ Archive".to_string(),
      reader,
      exec_path,
      response,
      |reader, unzip_path, response| {
        let name = &response.file_props.prefix;
        let file_path = unzip_path.join(name);

        let mut output_file = File::create(&file_path).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        let mut decoder = XzEncoder::new(reader, 9);

        io::copy(&mut decoder, &mut output_file).map_err(|err| {
          let err_msg = Error::Error(err.to_string()).to_string();
          error!("{},{},{}", file!(), line!(), err_msg);
          return err_msg;
        })?;

        Ok(())
      },
    )?;

    info!("success");
    Ok(res)
  }

  /// 7z
  pub fn prepare_7z(
    reader: BufReader<File>,
    exec_path: &PathBuf,
    response: HttpResponse,
  ) -> Result<HttpResponse, String> {
    let res = Self::decompress(
      "7Z Archive".to_string(),
      reader,
      exec_path,
      response,
      |_, unzip_path, response| {
        sevenz_rust::decompress_file(&Path::new(&response.file_props.path), unzip_path).map_err(
          |err| {
            let err_msg = Error::Error(err.to_string()).to_string();
            error!("{},{},{}", file!(), line!(), err_msg);
            return err_msg;
          },
        )?;

        Ok(())
      },
    )?;

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
          new_dir.suffix = if props.is_directory {
            String::new()
          } else {
            FileHandler::get_file_suffix(&name)
          };
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
