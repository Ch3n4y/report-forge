use crate::models::{ExcelProcessResult, LogLevel, LogMessage, ProgressInfo, ReportConfig};
use crate::processors::{ExcelProcessor, WordGenerator};
use anyhow::Result;
use std::sync::Mutex;
use tauri::State;

/// 应用状态
pub struct AppState {
    pub logs: Mutex<Vec<LogMessage>>,
    pub progress: Mutex<Option<ProgressInfo>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
            progress: Mutex::new(None),
        }
    }

    pub fn add_log(&self, level: LogLevel, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let log = LogMessage {
            level,
            message,
            timestamp,
        };

        if let Ok(mut logs) = self.logs.lock() {
            logs.push(log);
        }
    }

    pub fn update_progress(&self, current: usize, total: usize, message: String) {
        let percentage = if total > 0 {
            (current as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        let progress = ProgressInfo {
            current,
            total,
            message,
            percentage,
        };

        if let Ok(mut prog) = self.progress.lock() {
            *prog = Some(progress);
        }
    }

    pub fn clear_logs(&self) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
        }
    }

    pub fn clear_progress(&self) {
        if let Ok(mut prog) = self.progress.lock() {
            *prog = None;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// 处理Excel文件
#[tauri::command]
pub async fn process_excel_file(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<ExcelProcessResult, String> {
    state.add_log(LogLevel::Info, format!("开始处理Excel文件: {}", file_path));

    match ExcelProcessor::process_excel_to_json(&file_path) {
        Ok(result) => {
            state.add_log(
                LogLevel::Success,
                format!(
                    "Excel处理成功！共 {} 条记录，{} 个分组",
                    result.total_records, result.total_groups
                ),
            );
            Ok(result)
        }
        Err(e) => {
            let error_msg = format!("Excel处理失败: {}", e);
            state.add_log(LogLevel::Error, error_msg.clone());
            Err(error_msg)
        }
    }
}

/// 生成报告
#[tauri::command]
pub async fn generate_report(
    config: ReportConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    state.add_log(LogLevel::Info, "开始生成报告...".to_string());
    state.clear_progress();

    // 处理所有Excel文件
    let mut all_results = Vec::new();

    for (idx, excel_file) in config.excel_files.iter().enumerate() {
        state.update_progress(
            idx + 1,
            config.excel_files.len(),
            format!("正在处理Excel文件: {}", excel_file),
        );

        state.add_log(
            LogLevel::Info,
            format!("处理文件 {}/{}: {}", idx + 1, config.excel_files.len(), excel_file),
        );

        match ExcelProcessor::process_excel_to_json(excel_file) {
            Ok(result) => {
                state.add_log(
                    LogLevel::Success,
                    format!("文件处理成功: {} 条记录", result.total_records),
                );
                all_results.push(result);
            }
            Err(e) => {
                let error_msg = format!("处理文件失败 {}: {}", excel_file, e);
                state.add_log(LogLevel::Error, error_msg.clone());
                return Err(error_msg);
            }
        }
    }

    // 合并所有结果（如果有多个文件）
    let merged_result = if all_results.len() == 1 {
        all_results.into_iter().next().unwrap()
    } else {
        merge_excel_results(all_results)
    };

    state.update_progress(
        config.excel_files.len(),
        config.excel_files.len() + 1,
        "正在生成Word文档...".to_string(),
    );

    // 生成Word文档
    match WordGenerator::generate_report(&config, &merged_result) {
        Ok(output_file) => {
            state.add_log(
                LogLevel::Success,
                format!("报告生成成功！文件: {}", output_file),
            );
            state.update_progress(
                config.excel_files.len() + 1,
                config.excel_files.len() + 1,
                "完成！".to_string(),
            );
            Ok(output_file)
        }
        Err(e) => {
            let error_msg = format!("生成Word文档失败: {}", e);
            state.add_log(LogLevel::Error, error_msg.clone());
            Err(error_msg)
        }
    }
}

/// 获取日志
#[tauri::command]
pub async fn get_logs(state: State<'_, AppState>) -> Result<Vec<LogMessage>, String> {
    state
        .logs
        .lock()
        .map(|logs| logs.clone())
        .map_err(|e| format!("获取日志失败: {}", e))
}

/// 获取进度
#[tauri::command]
pub async fn get_progress(state: State<'_, AppState>) -> Result<Option<ProgressInfo>, String> {
    state
        .progress
        .lock()
        .map(|prog| prog.clone())
        .map_err(|e| format!("获取进度失败: {}", e))
}

/// 清空日志
#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.clear_logs();
    Ok(())
}

/// 清空进度
#[tauri::command]
pub async fn clear_progress(state: State<'_, AppState>) -> Result<(), String> {
    state.clear_progress();
    Ok(())
}

/// 合并多个Excel处理结果
fn merge_excel_results(results: Vec<ExcelProcessResult>) -> ExcelProcessResult {
    let mut total_records = 0;
    let mut all_grouped_data = Vec::new();

    for result in results {
        total_records += result.total_records;
        all_grouped_data.extend(result.grouped_data);
    }

    ExcelProcessResult {
        total_groups: all_grouped_data.len(),
        total_records,
        grouped_data: all_grouped_data,
    }
}
