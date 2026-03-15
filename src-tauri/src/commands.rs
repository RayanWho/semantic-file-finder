use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::AppState;

/// 搜索请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub directory: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub top_k: Option<usize>,
    pub threshold: Option<f32>,
}

/// 搜索结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub score: f32,
    pub summary: String,
    pub file_type: String,
    pub size: u64,
    pub modified: String,
}

/// 搜索响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query_time_ms: u64,
}

/// 索引状态
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexStatus {
    pub is_indexing: bool,
    pub indexed_files: usize,
    pub last_update: Option<String>,
    pub index_size_mb: f64,
}

/// 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub default_directory: Option<String>,
    pub excluded_dirs: Vec<String>,
    pub top_k: usize,
    pub threshold: f32,
    pub file_types: Vec<String>,
}

/// 搜索命令
#[tauri::command]
pub async fn search_files(
    request: SearchRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<SearchResponse, String> {
    let state = state.lock().await;
    
    // 调用搜索逻辑
    let results = state.search(&request.query)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(SearchResponse {
        results,
        total: results.len(),
        query_time_ms: 0, // TODO: 实际计算
    })
}

/// 开始索引
#[tauri::command]
pub async fn start_indexing(
    directory: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    
    state.start_indexing(&directory)
        .await
        .map_err(|e| e.to_string())
}

/// 获取索引状态
#[tauri::command]
pub async fn get_index_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<IndexStatus, String> {
    let state = state.lock().await;
    
    Ok(IndexStatus {
        is_indexing: state.is_indexing(),
        indexed_files: state.get_indexed_count(),
        last_update: state.get_last_update(),
        index_size_mb: state.get_index_size(),
    })
}

/// 更新配置
#[tauri::command]
pub async fn update_config(
    config: AppConfig,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.update_config(config);
    Ok(())
}

/// 获取配置
#[tauri::command]
pub async fn get_config(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AppConfig, String> {
    let state = state.lock().await;
    Ok(state.get_config().clone())
}

/// 选择目录
#[tauri::command]
pub async fn select_directory() -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog
    Ok(None)
}

/// 打开文件
#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    // TODO: 使用系统默认应用打开
    open::that(&path).map_err(|e| e.to_string())
}

/// 复制路径
#[tauri::command]
pub async fn copy_path(path: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    // TODO: 复制到剪贴板
    Ok(())
}
