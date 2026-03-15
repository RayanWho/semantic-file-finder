use anyhow::{Result, Context};
use serde_json::{json, Value};
use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::io::{Write, BufRead, BufReader};
use std::sync::Mutex;
use log::{info, error, warn};

/// Python Worker 管理器
pub struct PythonWorkerManager {
    embedding_worker: Mutex<Option<WorkerProcess>>,
    parser_worker: Mutex<Option<WorkerProcess>>,
    indexer_worker: Mutex<Option<WorkerProcess>>,
    worker_dir: String,
}

/// Worker 进程封装
pub struct WorkerProcess {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl PythonWorkerManager {
    /// 创建新的 Worker 管理器
    pub fn new(worker_dir: &str) -> Self {
        Self {
            embedding_worker: Mutex::new(None),
            parser_worker: Mutex::new(None),
            indexer_worker: Mutex::new(None),
            worker_dir: worker_dir.to_string(),
        }
    }

    /// 初始化所有 Workers
    pub fn init_workers(&self) -> Result<()> {
        info!("Initializing Python workers...");
        
        // 初始化 embedding worker
        self.start_embedding_worker()?;
        
        // 初始化 parser worker
        self.start_parser_worker()?;
        
        // 初始化 indexer worker
        self.start_indexer_worker()?;
        
        info!("All Python workers initialized");
        Ok(())
    }

    /// 启动 Embedding Worker
    fn start_embedding_worker(&self) -> Result<()> {
        let mut worker = self.start_worker("embedding_worker.py")?;
        
        // 发送初始化命令
        let request = json!({
            "action": "init",
            "model_path": "models"
        });
        self.send_request(&mut worker, &request)?;
        
        let response = self.receive_response(&mut worker)?;
        if response.get("status") == Some(&Value::String("loaded".to_string())) 
            || response.get("status") == Some(&Value::String("initialized".to_string())) {
            info!("Embedding worker initialized");
            let mut embedding = self.embedding_worker.lock().unwrap();
            *embedding = Some(worker);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to initialize embedding worker: {:?}", response))
        }
    }

    /// 启动 Parser Worker
    fn start_parser_worker(&self) -> Result<()> {
        let worker = self.start_worker("parser_worker.py")?;
        
        let mut parser = self.parser_worker.lock().unwrap();
        *parser = Some(worker);
        info!("Parser worker initialized");
        Ok(())
    }

    /// 启动 Indexer Worker
    fn start_indexer_worker(&self) -> Result<()> {
        let mut worker = self.start_worker("indexer_worker.py")?;
        
        // 发送初始化命令
        let request = json!({
            "action": "init",
            "index_dir": "index"
        });
        self.send_request(&mut worker, &request)?;
        
        let response = self.receive_response(&mut worker)?;
        if response.get("status") == Some(&Value::String("initialized".to_string())) {
            info!("Indexer worker initialized");
            let mut indexer = self.indexer_worker.lock().unwrap();
            *indexer = Some(worker);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to initialize indexer worker: {:?}", response))
        }
    }

    /// 启动通用 Worker 进程
    fn start_worker(&self, worker_name: &str) -> Result<WorkerProcess> {
        let worker_path = std::path::Path::new(&self.worker_dir).join(worker_name);
        
        let mut process = Command::new("python3")
            .arg(&worker_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(format!("Failed to start {}", worker_name))?;

        let stdin = process.stdin.take()
            .context(format!("Failed to open stdin for {}", worker_name))?;
        let stdout = process.stdout.take()
            .context(format!("Failed to open stdout for {}", worker_name))?;
        let stdout_reader = BufReader::new(stdout);

        Ok(WorkerProcess {
            process,
            stdin,
            stdout: stdout_reader,
        })
    }

    /// 发送请求到 Worker
    fn send_request(&self, worker: &mut WorkerProcess, request: &Value) -> Result<()> {
        let mut stdin = &worker.stdin;
        writeln!(stdin, "{}", request)?;
        stdin.flush()?;
        Ok(())
    }

    /// 从 Worker 接收响应
    fn receive_response(&self, worker: &mut WorkerProcess) -> Result<Value> {
        let mut line = String::new();
        worker.stdout.read_line(&mut line)?;
        
        if line.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty response from worker"));
        }

        let response: Value = serde_json::from_str(&line)
            .context(format!("Failed to parse response: {}", line))?;
        
        Ok(response)
    }

    /// 编码文本（单个）
    pub fn encode_text(&self, text: &str, is_query: bool) -> Result<Vec<f32>> {
        let request = json!({
            "action": "encode",
            "texts": [text],
            "is_query": is_query
        });

        let response = self.send_embedding_request(&request)?;
        
        let embeddings = response.get("embeddings")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid embedding response"))?;

        let embedding: Vec<f32> = embeddings.iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    /// 批量编码文本
    pub fn encode_texts(&self, texts: &[String], is_query: bool) -> Result<Vec<Vec<f32>>> {
        let request = json!({
            "action": "encode",
            "texts": texts,
            "is_query": is_query
        });

        let response = self.send_embedding_request(&request)?;
        
        let embeddings = response.get("embeddings")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid embedding response"))?;

        let result: Vec<Vec<f32>> = embeddings.iter()
            .map(|embedding| {
                embedding.as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            })
            .collect();

        Ok(result)
    }

    /// 发送请求到 Embedding Worker
    fn send_embedding_request(&self, request: &Value) -> Result<Value> {
        let mut embedding = self.embedding_worker.lock().unwrap();
        let worker = embedding.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Embedding worker not initialized"))?;
        
        self.send_request(worker, request)?;
        self.receive_response(worker)
    }

    /// 解析文件
    pub fn parse_file(&self, file_path: &str) -> Result<(String, Value)> {
        let request = json!({
            "action": "parse",
            "file_path": file_path
        });

        let mut parser = self.parser_worker.lock().unwrap();
        let worker = parser.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Parser worker not initialized"))?;
        
        self.send_request(worker, &request)?;
        let response = self.receive_response(worker)?;
        
        let content = response.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid parse response"))?
            .to_string();

        let metadata = response.get("metadata")
            .cloned()
            .unwrap_or(Value::Null);

        Ok((content, metadata))
    }

    /// 索引文件
    pub fn index_files(&self, files: &[Value]) -> Result<Value> {
        let request = json!({
            "action": "index",
            "files": files
        });

        let mut indexer = self.indexer_worker.lock().unwrap();
        let worker = indexer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Indexer worker not initialized"))?;
        
        self.send_request(worker, &request)?;
        let response = self.receive_response(worker)?;
        
        // 保存索引
        let save_request = json!({"action": "save"});
        self.send_request(worker, &save_request)?;
        self.receive_response(worker)?;
        
        Ok(response)
    }

    /// 搜索相似文件
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<Value>> {
        let request = json!({
            "action": "search",
            "query_embedding": query_embedding,
            "top_k": top_k
        });

        let mut indexer = self.indexer_worker.lock().unwrap();
        let worker = indexer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Indexer worker not initialized"))?;
        
        self.send_request(worker, &request)?;
        let response = self.receive_response(worker)?;
        
        let results = response.get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(results)
    }

    /// 获取索引统计
    pub fn get_index_stats(&self) -> Result<Value> {
        let request = json!({
            "action": "stats"
        });

        let mut indexer = self.indexer_worker.lock().unwrap();
        let worker = indexer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Indexer worker not initialized"))?;
        
        self.send_request(worker, &request)?;
        self.receive_response(worker)
    }

    /// 关闭所有 Workers
    pub fn shutdown(&self) {
        info!("Shutting down Python workers...");
        
        let workers = [
            ("embedding", self.embedding_worker.lock().unwrap().as_mut()),
            ("parser", self.parser_worker.lock().unwrap().as_mut()),
            ("indexer", self.indexer_worker.lock().unwrap().as_mut()),
        ];

        for (name, worker_opt) in workers.iter() {
            if let Some(worker) = worker_opt {
                let quit_request = json!({"action": "quit"});
                if let Ok(_) = self.send_request(worker, &quit_request) {
                    info!("{} worker shutdown", name);
                }
            }
        }
    }
}

impl Drop for PythonWorkerManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
