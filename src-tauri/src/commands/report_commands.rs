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

    // 先合并所有Excel文件
    state.update_progress(
        1,
        3,
        format!("正在合并 {} 个Excel文件...", config.excel_files.len()),
    );

    state.add_log(
        LogLevel::Info,
        format!("准备合并 {} 个Excel文件", config.excel_files.len()),
    );

    // 合并所有Excel文件，验证表头一致性
    let merged_data = match ExcelProcessor::merge_excel_files(&config.excel_files) {
        Ok(data) => {
            state.add_log(
                LogLevel::Success,
                format!("Excel文件合并成功！共 {} 行数据", data.rows.len()),
            );
            data
        }
        Err(e) => {
            let error_msg = format!("合并Excel文件失败: {}", e);
            state.add_log(LogLevel::Error, error_msg.clone());
            return Err(error_msg);
        }
    };

    // 处理合并后的数据（去重、分组）
    state.update_progress(
        2,
        3,
        "正在处理数据（去重、分组）...".to_string(),
    );

    state.add_log(LogLevel::Info, "开始处理合并后的数据...".to_string());

    let processed_result = match ExcelProcessor::process_raw_data(merged_data) {
        Ok(result) => {
            state.add_log(
                LogLevel::Success,
                format!(
                    "数据处理成功！共 {} 条记录，{} 个分组",
                    result.total_records, result.total_groups
                ),
            );
            result
        }
        Err(e) => {
            let error_msg = format!("数据处理失败: {}", e);
            state.add_log(LogLevel::Error, error_msg.clone());
            return Err(error_msg);
        }
    };

    // 生成Word文档
    state.update_progress(
        3,
        3,
        "正在生成Word文档...".to_string(),
    );

    match WordGenerator::generate_report(&config, &processed_result) {
        Ok(output_file) => {
            state.add_log(
                LogLevel::Success,
                format!("报告生成成功！文件: {}", output_file),
            );
            state.update_progress(
                3,
                3,
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
