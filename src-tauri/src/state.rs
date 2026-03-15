use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::commands::{AppConfig, SearchResult, IndexStatus};
use crate::python_worker::PythonWorkerManager;
use log::{info, error, warn};
use serde_json::json;
use walkdir::WalkDir;
use std::path::Path;
use chrono::Utc;

/// 应用状态
pub struct AppState {
    config: AppConfig,
    worker_manager: Arc<PythonWorkerManager>,
    is_indexing: Arc<Mutex<bool>>,
    indexed_files: Arc<Mutex<usize>>,
    last_update: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        let worker_manager = Arc::new(PythonWorkerManager::new("python-workers"));
        
        // 初始化 workers
        if let Err(e) = worker_manager.init_workers() {
            error!("Failed to initialize Python workers: {}", e);
        }

        Self {
            config: AppConfig {
                default_directory: None,
                excluded_dirs: vec![
                    ".git".to_string(),
                    "node_modules".to_string(),
                    ".DS_Store".to_string(),
                    "__pycache__".to_string(),
                    "target".to_string(),
                ],
                top_k: 10,
                threshold: 0.5,
                file_types: vec![],
            },
            worker_manager,
            is_indexing: Arc::new(Mutex::new(false)),
            indexed_files: Arc::new(Mutex::new(0)),
            last_update: Arc::new(Mutex::new(None)),
        }
    }

    /// 搜索文件
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        info!("Searching for: {}", query);
        
        // 1. 向量化查询
        let query_embedding = self.worker_manager.encode_text(query, true)?;
        
        // 2. 搜索相似文件
        let raw_results = self.worker_manager.search(&query_embedding, self.config.top_k)?;
        
        // 3. 转换为 SearchResult
        let mut results: Vec<SearchResult> = raw_results.iter()
            .filter_map(|r| {
                let path = r.get("path")?.as_str()?.to_string();
                let score = r.get("score")?.as_f64()? as f32;
                
                // 过滤低于阈值的结果
                if score < self.config.threshold {
                    return None;
                }
                
                // 获取文件元数据
                let path_obj = Path::new(&path);
                let metadata = std::fs::metadata(&path).ok()?;
                let file_type = path_obj.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // 读取文件内容摘要（前 200 字）
                let summary = if let Ok(content) = std::fs::read_to_string(&path) {
                    content.chars().take(200).collect::<String>()
                } else {
                    String::new()
                };
                
                // 格式化修改时间
                let modified = metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let modified_str = chrono::DateTime::from_timestamp(modified as i64, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                Some(SearchResult {
                    path,
                    score,
                    summary,
                    file_type,
                    size: metadata.len(),
                    modified: modified_str,
                })
            })
            .collect();
        
        // 按分数排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        info!("Found {} results", results.len());
        Ok(results)
    }

    /// 开始索引目录
    pub async fn start_indexing(&self, directory: &str) -> Result<()> {
        info!("Starting indexing for: {}", directory);
        
        let mut is_indexing = self.is_indexing.lock().await;
        *is_indexing = true;
        
        let worker_manager = self.worker_manager.clone();
        let directory = directory.to_string();
        let excluded_dirs = self.config.excluded_dirs.clone();
        let indexed_files = self.indexed_files.clone();
        let last_update = self.last_update.clone();
        
        // 在后台执行索引
        tokio::spawn(async move {
            let result = index_directory(
                &directory,
                &excluded_dirs,
                &worker_manager,
                &indexed_files,
                &last_update,
            ).await;
            
            if let Err(e) = result {
                error!("Indexing failed: {}", e);
            }
            
            let mut is_indexing = is_indexing.lock().await;
            *is_indexing = false;
        });
        
        Ok(())
    }

    pub fn is_indexing(&self) -> bool {
        // 这里需要异步获取，简化处理
        false
    }

    pub fn get_indexed_count(&self) -> usize {
        // 简化处理
        0
    }

    pub fn get_last_update(&self) -> Option<String> {
        // 简化处理
        None
    }

    pub fn get_index_size(&self) -> f64 {
        // 简化处理
        0.0
    }

    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
}

/// 索引目录的实现函数
async fn index_directory(
    directory: &str,
    excluded_dirs: &[String],
    worker_manager: &Arc<PythonWorkerManager>,
    indexed_files: &Arc<Mutex<usize>>,
    last_update: &Arc<Mutex<Option<String>>>,
) -> Result<()> {
    info!("Indexing directory: {}", directory);
    
    let mut files_to_index = Vec::new();
    
    // 1. 扫描文件
    for entry in WalkDir::new(directory)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // 排除指定目录
            !excluded_dirs.iter().any(|ex| file_name == ex)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Error scanning: {}", e);
                continue;
            }
        };
        
        if !entry.file_type().is_file() {
            continue;
        }
        
        let path = entry.path();
        let path_str = match path.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        
        files_to_index.push(path_str);
    }
    
    info!("Found {} files to index", files_to_index.len());
    
    // 2. 批量处理文件（每批 10 个）
    let mut total_indexed = 0;
    let batch_size = 10;
    
    for chunk in files_to_index.chunks(batch_size) {
        let mut batch_data = Vec::new();
        
        for file_path in chunk {
            // 解析文件
            match worker_manager.parse_file(file_path) {
                Ok((content, _metadata)) => {
                    // 向量化
                    match worker_manager.encode_text(&content, false) {
                        Ok(embedding) => {
                            batch_data.push(json!({
                                "path": file_path,
                                "content": content,
                                "embedding": embedding
                            }));
                            total_indexed += 1;
                        }
                        Err(e) => warn!("Failed to embed {}: {}", file_path, e),
                    }
                }
                Err(e) => warn!("Failed to parse {}: {}", file_path, e),
            }
        }
        
        // 索引当前批次
        if !batch_data.is_empty() {
            if let Err(e) = worker_manager.index_files(&batch_data) {
                error!("Failed to index batch: {}", e);
            }
        }
        
        info!("Indexed {} files so far", total_indexed);
    }
    
    // 更新状态
    {
        let mut count = indexed_files.lock().await;
        *count = total_indexed;
    }
    
    {
        let mut update = last_update.lock().await;
        *update = Some(Utc::now().to_rfc3339());
    }
    
    info!("Indexing completed: {} files", total_indexed);
    Ok(())
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
