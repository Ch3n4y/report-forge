mod commands;
mod models;
mod processors;

use commands::{
    clear_logs, clear_progress, generate_report, get_logs, get_progress, process_excel_file,
    AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::init();

    // 创建应用状态
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            process_excel_file,
            generate_report,
            get_logs,
            get_progress,
            clear_logs,
            clear_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
