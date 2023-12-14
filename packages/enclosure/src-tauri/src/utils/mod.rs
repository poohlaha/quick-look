use base64::Engine;
use std::path::{Path, PathBuf};

pub mod file;

pub struct Utils;

impl Utils {
    /// 获取解压目录
    pub fn get_program_dir() -> PathBuf {
        let path;
        let data_dir = dirs::data_dir();
        if data_dir.is_none() {
            path = dirs::home_dir().unwrap();
        } else {
            path = data_dir.unwrap()
        }

        return path.join(Path::new("QuickLook"));
    }

    /// 生成 base64 图片
    pub fn generate_image(data: Vec<u8>) -> String {
        let str = base64::engine::general_purpose::STANDARD.encode::<Vec<u8>>(data);
        let mut content = String::from("data:image/png;base64,");
        content.push_str(&str);
        return content;
    }
}
