//! pdf、doc、ppt预览

use crate::analysis::process::Process;
use crate::config::{HttpResponse, SuffixProps, DOCUMENT_SUFFIXES};
use crate::error::Error;
use crate::prepare::Prepare;
use crate::utils::file::FileUtils;
use crate::utils::Utils;
use log::info;
use mupdf::Matrix;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub struct Document;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PreviewProps {
    pub name: String,
    pub path: String,
    pub content: String,
}

impl Prepare<HttpResponse> for Document {
    fn with_response(response: HttpResponse) -> Result<HttpResponse, String> {
        let file_path = &response.file_props.path;
        let suffix = response.file_props.suffix.clone();

        // pdf
        let pdf = DOCUMENT_SUFFIXES.get(0).unwrap();

        // doc
        let doc = DOCUMENT_SUFFIXES.get(1).unwrap();

        // docx
        let docx = DOCUMENT_SUFFIXES.get(2).unwrap();

        if suffix.ends_with(pdf) {
            return Self::prepare_pdf(file_path, response.clone());
        }

        if suffix.ends_with(docx) {
            return Self::prepare_docx(file_path, response.clone());
        }

        Ok(response)
    }
}

impl Document {
    fn prepare<F>(file_path: &str, mut response: HttpResponse, func: F) -> Result<HttpResponse, String>
    where
        F: FnOnce(&str, &PathBuf, HttpResponse) -> Result<(), String>,
    {
        let temp_dir = FileUtils::create_temp_dir(&response.file_props.prefix, true)?;

        func(file_path, &temp_dir, response.clone())?;

        info!("read pictures ...");
        let contents = Self::read_pictures(&temp_dir)?;

        let suffix = response.file_props.suffix.clone();
        response.code = 200;
        response.body = serde_json::to_string(&contents).unwrap_or("".to_string());
        response.suffix_props = SuffixProps {
            name: response.file_props.suffix.clone(),
            _type: String::from("preview"),
            list: vec![suffix.clone()],
        };

        // 写入到 json 文件
        Process::copy_write_to_file(&temp_dir, &response)?;
        Ok(response)
    }

    /// pdf
    fn prepare_pdf(file_path: &str, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare pdf ...");

        let res = Self::prepare(file_path, response, |file_path, temp_dir, _| {
            let document = mupdf::document::Document::open(file_path).map_err(|err| Error::Error(err.to_string()).to_string())?;
            let pages = document.pages().map_err(|err| Error::Error(err.to_string()).to_string())?;

            for (i, page) in pages.enumerate() {
                let page = page.map_err(|err| Error::Error(err.to_string()).to_string())?;
                let matrix = Matrix::new_scale(1.0, 1.0);
                let pixmap = page
                    .to_pixmap(&matrix, &mupdf::Colorspace::device_rgb(), 0.0, true)
                    .map_err(|err| Error::Error(err.to_string()).to_string())?;

                let output_path = temp_dir.clone().join(&format!("page-{}.png", i));
                let output_dir = output_path.as_path().to_string_lossy().to_string();

                pixmap
                    .save_as(&output_dir, mupdf::ImageFormat::PNG)
                    .map_err(|err| Error::Error(err.to_string()).to_string())?;
            }

            Ok(())
        })?;

        info!("prepare pdf success !");
        Ok(res)
    }

    /// doc
    fn prepare_docx(file_path: &str, response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare docx ...");

        let res = Self::prepare(file_path, response, |file_path, temp_dir, _| {
            /*
            let file = FileUtils::read_file(file_path)?;
            let docx = docx_rs::read_docx(&file.to_vec()).map_err(|err| Error::Error(err.to_string()).to_string())?;
            let run = docx_rs::Run::new().add_break(docx_rs::BreakType::Page);
            let paragraph = Paragraph::new().add_run(run);

            let xml_docx = docx.clone().add_paragraph(paragraph).build();
            let document_rels = docx.document_rels;
            println!("document_rels: {:?}", document_rels);
            let images = xml_docx.media;
            if images.len() == 0 {
                return Err(Error::Error("failed to prepare docx".to_string()).to_string())
            }

            for (i, (_, bytes)) in images.iter().enumerate() {
                let output_path = temp_dir.clone().join(&format!("page-{}.png", i));
                let output_dir = output_path.as_path().to_string_lossy().to_string();
                fs::write(&output_dir, bytes).map_err(|err| Error::Error(err.to_string()).to_string())?;
            }

            */

            /*
            let docx = docx::DocxFile::from_file(PathBuf::from(file_path)).map_err(|_| Error::Error("parse docx error !".to_string()).to_string())?;
            let docx = docx.parse().map_err(|_| Error::Error("parse docx error !".to_string()).to_string())?;
            let document = docx.document;
            let app = docx.app.ok_or(Error::Error("parse docx error !".to_string()).to_string())?;
            let pages = app.pages.ok_or(Error::Error("get docx pages error !".to_string()).to_string())?;
            let pages = pages.parse::<i32>().map_err(|_| Error::Error("parse docx pages error !".to_string()).to_string())?;
            println!("pages: {:#?}", pages);
             */

            /*
            let contents = document.body.content;

            for (i, content) in contents.iter().enumerate() {
                if let docx::document::BodyContent::Paragraph(paragraph) = content {
                    println!("paragraph: {:#?}", paragraph);
                   for paragraph_content in paragraph.content.iter() {
                       println!("paragraph_content: {:#?}", paragraph_content);
                       if let docx::document::ParagraphContent::Run(run) = paragraph_content {
                           println!("run: {:#?}", run);
                           let run_content = &run.content;
                           for content  in run_content.iter() {
                               println!("run content: {:#?}", content);
                               if let docx::document::RunContent::Break(r_break) = content {
                                   println!("break: {:#?}", r_break);
                               }
                           }
                       }
                   }
                }
            }
             */
            Ok(())
        })?;

        info!("prepare docx success !");
        Ok(res)
    }

    /// 读取图片转成 base64
    fn read_pictures(file_path: &PathBuf) -> Result<Vec<PreviewProps>, String> {
        let mut contents: Vec<PreviewProps> = Vec::new();
        let entries = fs::read_dir(&file_path).map_err(|err| Error::Error(err.to_string()).to_string())?;

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let filename = entry.file_name().to_str().unwrap_or("").to_string();
            if !filename.starts_with("page-") {
                continue;
            }

            let content = FileUtils::read_file(&path_str)?;
            let content = Utils::generate_image(content);
            contents.push(PreviewProps {
                name: filename,
                path: path_str,
                content,
            })
        }

        return Ok(contents);
    }
}
