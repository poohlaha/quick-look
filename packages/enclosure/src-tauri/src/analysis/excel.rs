//! 读取 excel

use std::env::temp_dir;
use crate::config::{ExcelCell, ExcelRow, ExcelSheet, ExcelSheetMetadata, HttpResponse, MAX_ASYNC_TASK_COUNT, PREVIEW_FILE};
use crate::error::Error;
use crate::prepare::Prepare;
use crate::semaphore::Semaphore;
use crate::utils::file::FileUtils;
use async_std::sync::Arc;
use calamine::{open_workbook_auto, DataType, Range, Reader};
use log::{error, info};
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

        let mut index: u32 = 1;
        for sheet_name in sheets {
            info!("found sheet name `{}`", &sheet_name);
            let start_time = Instant::now();
            let work_range = workbook
                .worksheet_range(&sheet_name)
                .map_err(|err| Error::Error(err.to_string()).to_string())?;
            let temp_path_cloned = Arc::new(temp_path.clone());
            let res_cloned = Arc::new(res.clone());
            let index_cloned = Arc::new(index.clone());
            async_std::task::spawn_blocking(move || {
                let _ = Self::handle_sheet(work_range.clone(), &sheet_name, start_time.clone(), &*temp_path_cloned, &*res_cloned, *index_cloned);
            });

            index += 1;
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
        index: u32
    ) -> Result<(), String> {
        info!("handle sheet: {}", sheet_name);
        let sheet_index = index;

        // 计算时间
        let rows = range.rows();
        let (row_size, cell_size) = range.get_size();

        let elapsed_time = format!("{:.2?}", start_time.elapsed());
        info!("handle sheet: {}, row count: {}, cell count: {}, time: {}", sheet_name, row_size, cell_size, elapsed_time);

        let mut chunks = Vec::new();
        if row_size < TAKE_EXCEL_COUNT {
            for row in rows {
                chunks.push(row.to_owned());
            }

            let file_path = temp_path
                .clone()
                .join(&format!("{}-{}-{}", index, sheet_name, &response.file_props.prefix));
            println!("file_path: {:#?}", &file_path);
            let _ = Self::get_row(&chunks, 1, file_path)?;
            chunks.clear();

            let mut metadata = ExcelSheetMetadata::default();
            metadata.name = sheet_name.to_string();
            metadata.row_start = 1;
            metadata.row_end = row_size;
            metadata.task_id = 0;
            Self::write_result_to_file(temp_path.clone(), sheet_name.to_string(), sheet_index, row_size, cell_size, vec![metadata])?;
        } else {
            let semaphore = Arc::new(Semaphore::new(MAX_ASYNC_TASK_COUNT));
            let mut index = 0;
            let mut task_index = 0;
            let mut tasks = Vec::new();
            let mut metadatas: Vec<ExcelSheetMetadata> = Vec::new();
            let mut start_index = 1;

            // 大于 TAKE_EXCEL_COUNT 时启动异步任务
            for row in rows {
                index += 1;
                chunks.push(row.to_owned());

                if (chunks.len() == TAKE_EXCEL_COUNT && index != row_size) || (index == row_size) {
                    task_index += 1;
                    info!("exec task{}", task_index);
                    let file_path = temp_path.clone().join(&format!("{}-{}-{}-{}-{}", index, task_index, sheet_name, &response.file_props.prefix, task_index));
                    let semaphore_cloned = semaphore.clone();
                    let chunks_clone = Arc::new(chunks.clone());
                    let index_clone = Arc::new(task_index.clone());
                    let temp_path_clone = Arc::new(file_path.clone());
                    let res_clone = Arc::new(response.clone());
                    let result = async_std::task::spawn(Self::handle_row(semaphore_cloned, chunks_clone, index_clone, temp_path_clone, res_clone));
                    tasks.push(result);


                    let mut metadata = ExcelSheetMetadata::default();
                    metadata.name = sheet_name.to_string();
                    metadata.row_start = start_index;
                    metadata.row_end = index;
                    metadata.task_id = task_index;

                    if index == row_size {
                        start_index = 1
                    } else {
                        start_index = index + 1;
                    }

                    metadatas.push(metadata);
                    chunks.clear();
                }
            }

            let sheet_name = sheet_name.to_string();
            let temp_path = temp_path.clone();
           async_std::task::spawn(async move {
               match futures::future::try_join_all(tasks).await {
                    Ok(_) => {
                        info!("execute tasks success !");
                        // 完成所有任务后写入数据
                        Self::write_result_to_file(
                            temp_path,
                            sheet_name.clone(),
                            sheet_index,
                            row_size,
                            cell_size,
                            metadatas
                        ).ok();
                    }
                    Err(err) => {
                        error!("execute tasks error: {}", err);
                    }
                }
            });
        }

        // 任务完成后发送消息到前端
        Ok(())
    }

    /// 写入结果
    fn write_result_to_file(temp_dir: PathBuf, sheet_name: String, index: u32, rows_count: usize, cells_count: usize, metadata: Vec<ExcelSheetMetadata>) -> Result<(), String> {
        let mut sheet = ExcelSheet::default();
        sheet.name = sheet_name;
        sheet.rows_count = rows_count;
        sheet.cells_count = cells_count;
        sheet.metadata = metadata;
        sheet.index = index;

        // 写入到文件
        info!("write sheet info into json ...");

        // read file content
        let json_file_path = temp_dir.join(PREVIEW_FILE);
        let json_file_str = json_file_path.to_string_lossy().to_string();

        let mut sheets: Vec<ExcelSheet> = Vec::new();
        if json_file_path.exists() {
            let content = FileUtils::read_file_string(&json_file_str)?;

            if !content.is_empty() {
                sheets = serde_json::from_str(&content).map_err(|err| Error::Error(err.to_string()).to_string())?;
            }
        }

        sheets.push(sheet);
        let content = serde_json::to_string_pretty(&sheets).unwrap(); // 序列化为漂亮格式的 JSON 字符串
        FileUtils::write_to_file_when_clear(&json_file_str, &content)?;
        info!("write info json file success !");
        Ok(())
    }

    /// 处理行
    async fn handle_row(
        semaphore: Arc<Semaphore>,
        chunks: Arc<Vec<Vec<DataType>>>,
        index: Arc<usize>,
        temp_path: Arc<PathBuf>,
        response: Arc<HttpResponse>,
    ) -> Result<(), String> {
        let index = *index;
        info!("handle row, task {} ...", index);
        semaphore.acquire().await;

        let chunks: &Vec<Vec<DataType>> = &*chunks;

        // file_path
        let file_path = &*temp_path;
        Self::get_row(chunks, index, file_path.clone())?;
        semaphore.release().await;
        Ok(())
    }

    fn get_row(chunks: &Vec<Vec<DataType>>, index: usize, file_path: PathBuf) -> Result<(), String> {
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
                        error!("read row: {} cell: {}, error: {:#?}", x, y, err);
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
        let content = serde_json::to_string(&rows).unwrap(); // 序列化为漂亮格式的 JSON 字符串
        FileUtils::write_to_file_when_clear(&file_path, &content)?;
        info!("generate file: {}", &file_path);

        Ok(())
    }
}
