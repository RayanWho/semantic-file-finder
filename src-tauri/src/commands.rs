use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use std::process::{Command, Stdio};
use tokio::sync::Mutex;
use tauri_plugin_dialog::DialogExt;
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
    pub indexed_target: usize,
    pub last_update: Option<String>,
    pub index_size_mb: f64,
    pub default_directory: Option<String>,
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
    let results: Vec<SearchResult> = state.search(
        &request.query,
        request.top_k,
        request.threshold,
        request.file_types.as_deref(),
        request.directory.as_deref(),
    )
        .await
        .map_err(|e: anyhow::Error| e.to_string())?;
    
    let total = results.len();
    
    Ok(SearchResponse {
        results,
        total,
        query_time_ms: 0, // TODO: 实际计算
    })
}

/// 开始索引
#[tauri::command]
pub async fn start_indexing(
    directory: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    
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
        indexed_target: state.get_index_target(),
        last_update: state.get_last_update(),
        index_size_mb: state.get_index_size(),
        default_directory: state.get_config().default_directory.clone(),
    })
}

/// 更新配置
#[tauri::command]
pub async fn update_config(
    config: AppConfig,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.update_config(config).map_err(|e| e.to_string())
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
pub async fn select_directory(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let selected = app_handle.dialog().file().blocking_pick_folder();
    match selected {
        Some(path) => Ok(Some(
            path.into_path()
                .map_err(|e| e.to_string())?
                .display()
                .to_string(),
        )),
        None => Ok(None),
    }
}

/// 打开文件
#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    // TODO: 使用系统默认应用打开
    open::that(&path).map_err(|e| e.to_string())
}

/// 复制路径
#[tauri::command]
pub async fn copy_path(path: String, _app_handle: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        child.stdin.as_mut()
            .ok_or_else(|| "Failed to open clipboard stdin".to_string())?
            .write_all(path.as_bytes())
            .map_err(|e| e.to_string())?;
        child.wait().map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let mut child = Command::new("clip")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        child.stdin.as_mut()
            .ok_or_else(|| "Failed to open clipboard stdin".to_string())?
            .write_all(path.as_bytes())
            .map_err(|e| e.to_string())?;
        if child.wait().map_err(|e| e.to_string())?.success() {
            return Ok(());
        }
        return Err("Failed to copy path".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = [("wl-copy", vec![]), ("xclip", vec!["-selection", "clipboard"])];
        for (program, args) in candidates {
            if let Ok(mut child) = Command::new(program)
                .args(args)
                .stdin(Stdio::piped())
                .spawn()
            {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(path.as_bytes()).map_err(|e| e.to_string())?;
                }
                let status = child.wait().map_err(|e| e.to_string())?;
                if status.success() {
                    return Ok(());
                }
            }
        }
        return Err("Failed to copy path: wl-copy/xclip unavailable".to_string());
    }

    #[allow(unreachable_code)]
    Err("Clipboard copy is not supported on this platform".to_string())
}
