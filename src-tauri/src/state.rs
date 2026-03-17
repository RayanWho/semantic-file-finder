use anyhow::Result;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::commands::{AppConfig, SearchResult};
use crate::python_worker::PythonWorkerManager;
use log::{info, error, warn};
use serde_json::json;
use walkdir::WalkDir;
use chrono::Utc;

/// 应用状态
pub struct AppState {
    config: AppConfig,
    config_path: PathBuf,
    worker_manager: Arc<PythonWorkerManager>,
    is_indexing: Arc<AtomicBool>,
    indexed_files: Arc<AtomicUsize>,
    index_target: Arc<AtomicUsize>,
    last_update: Arc<StdMutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (worker_dir, model_dir, index_dir) = PythonWorkerManager::project_paths();
        let worker_manager = Arc::new(PythonWorkerManager::new(worker_dir, model_dir, index_dir));
        let config_path = default_config_path();
        
        // 初始化 workers
        if let Err(e) = worker_manager.init_workers() {
            error!("Failed to initialize Python workers: {}", e);
        }

        let config = load_config(&config_path);
        let indexed_files = Arc::new(AtomicUsize::new(0));
        let index_target = Arc::new(AtomicUsize::new(0));
        let last_update = Arc::new(StdMutex::new(None));

        if let Ok(stats) = worker_manager.get_index_stats() {
            if let Some(count) = stats.get("indexed_files").and_then(|v| v.as_u64()) {
                indexed_files.store(count as usize, Ordering::Relaxed);
            }
        }

        Self {
            config,
            config_path,
            worker_manager,
            is_indexing: Arc::new(AtomicBool::new(false)),
            indexed_files,
            index_target,
            last_update,
        }
    }

    /// 搜索文件
    pub async fn search(
        &self,
        query: &str,
        top_k: Option<usize>,
        threshold: Option<f32>,
        file_types: Option<&[String]>,
        directory: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        info!("Searching for: {}", query);

        if self.indexed_files.load(Ordering::Relaxed) == 0 {
            return Ok(Vec::new());
        }
        
        // 1. 向量化查询
        let query_embedding = self.worker_manager.encode_text(query, true)?;
        
        // 2. 搜索相似文件
        let raw_results = self.worker_manager.search(&query_embedding, top_k.unwrap_or(self.config.top_k))?;
        let threshold = threshold.unwrap_or(self.config.threshold);
        let file_types = file_types.unwrap_or(&self.config.file_types);
        
        // 3. 转换为 SearchResult
        let mut results: Vec<SearchResult> = raw_results.iter()
            .filter_map(|r| {
                let path = r.get("path")?.as_str()?.to_string();
                let score = r.get("score")?.as_f64()? as f32;

                if let Some(directory) = directory {
                    if !path.starts_with(directory) {
                        return None;
                    }
                }
                
                // 过滤低于阈值的结果
                if score < threshold {
                    return None;
                }
                
                // 获取文件元数据
                let path_obj = Path::new(&path);
                let metadata = std::fs::metadata(&path).ok()?;
                let file_type = path_obj.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                if !file_types.is_empty() {
                    let normalized_file_type = format!(".{}", file_type.to_lowercase());
                    let matches = file_types.iter().any(|t| {
                        let normalized = t.trim().to_lowercase();
                        normalized == normalized_file_type || normalized == file_type.to_lowercase()
                    });
                    if !matches {
                        return None;
                    }
                }
                
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

        if self.is_indexing.swap(true, Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Indexing is already in progress"));
        }

        self.indexed_files.store(0, Ordering::Relaxed);
        self.index_target.store(0, Ordering::Relaxed);
        self.is_indexing.store(true, Ordering::Relaxed);

        self.worker_manager.reset_index()?;
        
        let worker_manager = self.worker_manager.clone();
        let directory = directory.to_string();
        let excluded_dirs = self.config.excluded_dirs.clone();
        let file_types = self.config.file_types.clone();
        let indexed_files = self.indexed_files.clone();
        let index_target = self.index_target.clone();
        let last_update = self.last_update.clone();
        let is_indexing = self.is_indexing.clone();
        
        // 在后台执行索引
        tokio::spawn(async move {
            let result = index_directory(
                &directory,
                &excluded_dirs,
                &file_types,
                &worker_manager,
                &indexed_files,
                &index_target,
                &last_update,
            ).await;
            
            if let Err(e) = result {
                error!("Indexing failed: {}", e);
            }
            
            is_indexing.store(false, Ordering::Relaxed);
        });
        
        Ok(())
    }

    pub fn is_indexing(&self) -> bool {
        self.is_indexing.load(Ordering::Relaxed)
    }

    pub fn get_indexed_count(&self) -> usize {
        self.indexed_files.load(Ordering::Relaxed)
    }

    pub fn get_index_target(&self) -> usize {
        self.index_target.load(Ordering::Relaxed)
    }

    pub fn get_last_update(&self) -> Option<String> {
        self.last_update.lock().ok().and_then(|value| value.clone())
    }

    pub fn get_index_size(&self) -> f64 {
        let (_, _, index_dir) = PythonWorkerManager::project_paths();
        directory_size_mb(&index_dir)
    }

    pub fn update_config(&mut self, config: AppConfig) -> Result<()> {
        self.config = config;
        save_config(&self.config_path, &self.config)
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
}

/// 索引目录的实现函数
async fn index_directory(
    directory: &str,
    excluded_dirs: &[String],
    file_types: &[String],
    worker_manager: &Arc<PythonWorkerManager>,
    indexed_files: &Arc<AtomicUsize>,
    index_target: &Arc<AtomicUsize>,
    last_update: &Arc<StdMutex<Option<String>>>,
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
        if !file_types.is_empty() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            let normalized_ext = format!(".{}", ext);
            let matches = file_types.iter().any(|t| {
                let normalized = t.trim().to_lowercase();
                normalized == normalized_ext || normalized == ext
            });
            if !matches {
                continue;
            }
        }

        let path_str = match path.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        
        files_to_index.push(path_str);
    }
    
    info!("Found {} files to index", files_to_index.len());
    index_target.store(files_to_index.len(), Ordering::Relaxed);
    
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

        indexed_files.store(total_indexed, Ordering::Relaxed);
        
        info!("Indexed {} files so far", total_indexed);
    }
    
    // 更新状态
    indexed_files.store(total_indexed, Ordering::Relaxed);
    if let Ok(mut update) = last_update.lock() {
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

fn default_config() -> AppConfig {
    AppConfig {
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
    }
}

fn default_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("semantic-file-finder");
    path.push("config.json");
    path
}

fn load_config(path: &Path) -> AppConfig {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| default_config()),
        Err(_) => default_config(),
    }
}

fn save_config(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(config)?)?;
    Ok(())
}

fn directory_size_mb(path: &Path) -> f64 {
    let total_bytes: u64 = WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .map(|metadata| metadata.len())
        .sum();
    total_bytes as f64 / 1024.0 / 1024.0
}
