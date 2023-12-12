//! pdf、doc、ppt预览

use std::fs;
use std::path::PathBuf;
use log::{error, info};
use mupdf::Matrix;
use serde::{Deserialize, Serialize};
use crate::analysis::{HttpResponse, SuffixProps};
use crate::file::{FileHandler, OTHER_SUFFIX};
use crate::error::Error;

pub struct Preview;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PreviewProps {
    pub name: String,
    pub path: String,
    pub content: String,
}

impl Preview {

    pub fn exec(response: HttpResponse) -> Result<HttpResponse, String> {
        let suffix = response.file_props.suffix.clone();

        // pdf
        let pdf = OTHER_SUFFIX.get(0).unwrap();

        if suffix.ends_with(pdf) {
            return Self::prepare_pdf(&response.file_props.path, response.clone());
        }

        Ok(response)
    }

    fn prepare_pdf(file_path: &str, mut response: HttpResponse) -> Result<HttpResponse, String> {
        let document = mupdf::document::Document::open(file_path).map_err(|err| {
            let err_msg = Error::Error(err.to_string()).to_string();
            error!("{},{},{}", file!(), line!(), err_msg);
            return err_msg;
        })?;

        let pages = document.pages().map_err(|err| {
            let err_msg = Error::Error(err.to_string()).to_string();
            error!("{},{},{}", file!(), line!(), err_msg);
            return err_msg;
        })?;

        let temp_dir = FileHandler::create_temp_dir(&response.file_props.prefix)?;
        for (i, page) in pages.enumerate() {
            let page = page.map_err(|err| {
                let err_msg = Error::Error(err.to_string()).to_string();
                error!("{},{},{}", file!(), line!(), err_msg);
                return err_msg;
            })?;

            let matrix = Matrix::new_scale(1.0, 1.0);
            let pixmap = page
                .to_pixmap(&matrix, &mupdf::Colorspace::device_rgb(), 0.0, true)
                .map_err(|err| {
                    let err_msg = Error::Error(err.to_string()).to_string();
                    error!("{},{},{}", file!(), line!(), err_msg);
                    return err_msg;
                })?;

            let output_path = temp_dir.join(&format!("page-{}.png", i));
            let output_dir = output_path.as_path().to_string_lossy().to_string();

            println!("output_dir: {}", &output_dir);
            pixmap.save_as(&output_dir, mupdf::ImageFormat::PNG).map_err(|err| {
                let err_msg = Error::Error(err.to_string()).to_string();
                error!("{},{},{}", file!(), line!(), err_msg);
                return err_msg;
            })?;
        }

        let contents = Self::read_pictures(&temp_dir)?;
        let pdf = OTHER_SUFFIX.get(0).unwrap();

        response.code = 200;
        response.body = serde_json::to_string(&contents).unwrap_or("".to_string());
        response.suffix_props = SuffixProps {
            name: response.file_props.suffix.clone(),
            _type: String::from("preview"),
            list: vec![pdf.to_string()],
        };

        info!("success");
        return Ok(response);

    }

    /// 读取图片转成 base64
    fn read_pictures(file_path: &PathBuf) -> Result<Vec<PreviewProps>, String> {
        let mut contents: Vec<PreviewProps> = Vec::new();
        let entries = fs::read_dir(&file_path).map_err(|err| {
            let err_msg = Error::Error(err.to_string()).to_string();
            error!("{},{},{}", file!(), line!(), err_msg);
            return err_msg;
        })?;

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let filename = entry.file_name().to_str().unwrap_or("").to_string();
            if !filename.starts_with("page-") {
                continue;
            }

            let content = FileHandler::read_file(&path_str)?;
            let content = FileHandler::generate_image(content);
            contents.push(PreviewProps {
                name: filename,
                path: path_str,
                content,
            })
        }

        return Ok(contents);
    }


}