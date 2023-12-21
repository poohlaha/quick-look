//! configs

use crate::prepare::HttpResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 图片后缀
pub const IMAGE_SUFFIXES: [&str; 11] = ["jpeg", "jpg", "png", "gif", "tiff", "tif", "webp", "ico", "heic", "bmp", "svg"];

/// 文档后缀
pub const DOCUMENT_SUFFIXES: [&str; 5] = ["pdf", "doc", "docx", "ppt", "pptx"];

pub const EXCEL_SUFFIXES: [&str; 6] = ["xls", "xlsx", "xlsm", "xlsb", "xla", "xlam"];

/// 压缩包后缀
pub const ARCHIVE_SUFFIXES: [&str; 10] = ["zip", "bz2", "gz", "zlib", "tar", "rar", "7z", "tar.xz", "xz", "tgz"];

/// 预览文件
pub const PREVIEW_FILE: &str = "preview.json";

// history
pub const HISTORY_FILE: &str = "history";

// history count
pub const HISTORY_COUNT: usize = 30;

// 最大异步线程数
pub const MAX_ASYNC_TASK_COUNT: usize = 10;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub id: String,
    pub name: String,
    pub path: String,
    pub update_time: i64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SuffixProps {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub list: Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileProps {
    pub key: String,
    pub name: String,
    pub suffix: String,
    pub prefix: String,
    pub path: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    pub kind: String,
    pub size: String,
    pub old_size: u64,
    pub packed: String,
    pub modified: String,
    pub permissions: String,
    pub executable: bool,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    pub files: Vec<FileProps>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub(crate) code: u16,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
    #[serde(rename = "fileProps")]
    pub(crate) file_props: FileProps,
    pub(crate) error: String,
    #[serde(rename = "suffixProps")]
    pub(crate) suffix_props: SuffixProps,
}

impl HttpResponseData for HttpResponse {}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ExcelResponse {
    pub total_rows: usize,
    pub total_cells: usize,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ExcelRow {
    pub index: usize,
    pub cells: Vec<ExcelCell>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ExcelCell {
    pub value: String,
    pub row_index: usize,
    pub cell_index: usize,
}
