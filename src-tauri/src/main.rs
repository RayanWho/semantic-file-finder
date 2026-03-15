// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod python_worker;

use anyhow::Result;
use log::info;
use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() -> Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("Starting Semantic File Finder...");

    // 创建应用状态
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // 启动 Tauri 应用
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::search_files,
            commands::start_indexing,
            commands::get_index_status,
            commands::update_config,
            commands::get_config,
            commands::select_directory,
            commands::open_file,
            commands::copy_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
