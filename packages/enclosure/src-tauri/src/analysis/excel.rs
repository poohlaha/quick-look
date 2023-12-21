//! 读取 excel

use crate::analysis::process::Process;
use crate::config::{ExcelCell, ExcelRow, HttpResponse, MAX_ASYNC_TASK_COUNT};
use crate::error::Error;
use crate::prepare::Prepare;
use crate::semaphore::Semaphore;
use crate::utils::file::FileUtils;
use async_std::sync::Arc;
use calamine::{open_workbook, open_workbook_auto, DataType, Range, Reader, Sheets, Xlsx, XlsxError};
use log::{error, info};
use std::fs::File;
use std::io::BufReader;
use std::ops::Index;
use std::path::PathBuf;
use std::time::Instant;

pub struct Excel;

const TAKE_EXCEL_COUNT: usize = 100000;

impl Prepare<HttpResponse> for Excel {
    fn with_response(response: HttpResponse) -> Result<HttpResponse, String> {
        info!("prepare excel ...");

        // temp dir
        let temp_path = FileUtils::create_temp_dir(&response.file_props.prefix, true)?;
        let mut workbook = open_workbook_auto(&response.file_props.path).map_err(|err| Error::Error(err.to_string()).to_string())?;

        let mut res = response.clone();
        res.code = 200;
        // sheets
        let sheets = workbook.sheet_names().to_owned();
        if sheets.is_empty() {
            return Ok(res);
        }

        for sheet_name in sheets {
            info!("found sheet name `{}`", &sheet_name);
            let start_time = Instant::now();
            let work_range = workbook
                .worksheet_range(&sheet_name)
                .map_err(|err| Error::Error(err.to_string()).to_string())?;
            let temp_path_cloned = Arc::new(temp_path.clone());
            let res_cloned = Arc::new(res.clone());
            async_std::task::spawn_blocking(move || {
                Self::handle_sheet(work_range.clone(), &sheet_name, start_time.clone(), &*temp_path_cloned, &*res_cloned).unwrap();
            });
        }

        Ok(res)
    }
}

impl Excel {
    /// 处理单张 sheet
    fn handle_sheet(
        range: Range<DataType>,
        sheet_name: &str,
        start_time: Instant,
        temp_path: &PathBuf,
        response: &HttpResponse,
    ) -> Result<(), String> {
        info!("handle sheet: {}", sheet_name);

        // 计算时间
        let rows = range.rows();
        let cells = range.cells();
        let (row_size, cell_size) = range.get_size();
        info!("sheet name: {}, row_size: {}, cell_size: {}", sheet_name, row_size, cell_size);
        let total_count = rows.len();

        let elapsed_time = format!("{:.2?}", start_time.elapsed());
        info!("handle sheet: {},  count: {}, time: {}", sheet_name, total_count, elapsed_time);

        let mut chunks = Vec::new();
        if total_count < TAKE_EXCEL_COUNT {
            for row in rows {
                chunks.push(row.to_owned());
            }

            let file_path = temp_path
                .clone()
                .join(&response.file_props.prefix)
                .with_extension(&response.file_props.prefix);
            Self::get_row(&chunks, 0, file_path)?;
            chunks.clear();
        } else {
            let semaphore = Arc::new(Semaphore::new(MAX_ASYNC_TASK_COUNT));
            let mut index = 0;

            // 大于 TAKE_EXCEL_COUNT 时启动异步任务
            for row in rows {
                index += 1;
                chunks.push(row.to_owned());

                if (chunks.len() == TAKE_EXCEL_COUNT && index != total_count) || (index == total_count) {
                    let semaphore_cloned = semaphore.clone();
                    let chunks_clone = Arc::new(chunks.clone());
                    let index_clone = Arc::new(index.clone());
                    let temp_path_clone = Arc::new(temp_path.clone());
                    let res_clone = Arc::new(response.clone());
                    async_std::task::spawn(async move {
                        Self::handle_row(semaphore_cloned, chunks_clone, index_clone, temp_path_clone, res_clone).await;
                    });
                    chunks.clear();
                    index = 0;
                }
            }
        }

        // 任务完成后发送消息到前端
        Ok(())
    }

    /// 处理行
    async fn handle_row(
        semaphore: Arc<Semaphore>,
        chunks: Arc<Vec<Vec<DataType>>>,
        index: Arc<usize>,
        temp_path: Arc<PathBuf>,
        response: Arc<HttpResponse>,
    ) {
        let index = *index;
        info!("handle row, thread {} ...", index);
        semaphore.acquire().await;

        let chunks: &Vec<Vec<DataType>> = &*chunks;

        // file_path
        let file_path = temp_path.join(&response.file_props.prefix).with_extension(&format!("data-{index}"));
        Self::get_row(chunks, index, file_path);
        semaphore.release().await;
    }

    fn get_row(chunks: &Vec<Vec<DataType>>, index: usize, file_path: PathBuf) -> Result<bool, String> {
        let mut rows: Vec<ExcelRow> = Vec::new();

        for (x, chunk) in chunks.iter().enumerate() {
            let mut cells: Vec<ExcelCell> = Vec::new();
            for (y, c) in chunk.iter().enumerate() {
                let value = match c {
                    DataType::Empty => String::new(),
                    DataType::String(ref s) | DataType::DateTimeIso(ref s) | DataType::DurationIso(ref s) => String::from(s),
                    DataType::Float(ref f) | DataType::DateTime(ref f) | DataType::Duration(ref f) => f.to_string(),
                    DataType::Int(ref i) => i.to_string(),
                    DataType::Bool(ref b) => b.to_string(),
                    DataType::Error(err) => {
                        error!("read row error: {:#?}", err);
                        String::new()
                    }
                };

                cells.push(ExcelCell {
                    value,
                    row_index: x,
                    cell_index: y,
                });
            }

            rows.push(ExcelRow { index: x, cells })
        }

        // 数据处理完成后写入到文件
        let file_path = file_path.as_path().to_string_lossy().to_string();
        let content = serde_json::to_string_pretty(&rows).unwrap(); // 序列化为漂亮格式的 JSON 字符串
        FileUtils::write_to_file_when_clear(&file_path, &content)?;

        Ok(true)
    }
}
