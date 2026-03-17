#!/usr/bin/env python3
"""
Indexer Worker - usearch 向量索引管理
负责文件的索引创建、更新、查询

从 FAISS 迁移到 usearch:
- usearch 更快 (HNSW 实现优化)
- 内存占用更小
- API 更简洁
"""

import sys
import json
import logging
import hashlib
import pickle
import time
from typing import Dict, Any, List, Optional, Tuple
from pathlib import Path
from datetime import datetime

try:
    import usearch.index as ui
    import numpy as np
except ImportError as e:
    print(json.dumps({"error": f"Missing dependency: {e}. Install with: pip install usearch"}))
    sys.exit(1)

logger = logging.getLogger("indexer_worker")


class FileIndex:
    """文件索引管理器 - usearch 版本"""
    
    def __init__(self, index_dir: str = "index", dimension: int = 1024):
        self.index_dir = Path(index_dir)
        self.dimension = dimension
        self.index: Optional[ui.Index] = None
        self.file_metadata: Dict[str, Dict] = {}  # file_hash -> metadata
        self.file_paths: List[str] = []  # index ID -> file path
        self.loaded = False
        
        # 创建索引目录
        self.index_dir.mkdir(parents=True, exist_ok=True)
    
    def create_index(self):
        """创建 usearch 索引"""
        # usearch 使用 HNSW 算法
        # connectivity: M 参数，控制图的连通性 (默认 16)
        self.index = ui.Index(
            ndim=self.dimension,
            metric='cos'  # 余弦相似度 (等价于归一化后的内积)
        )
        logger.info(f"Created usearch HNSW index (dim={self.dimension})")
    
    def add_vectors(self, vectors: np.ndarray, file_paths: List[str], file_hashes: List[str]):
        """添加向量到索引"""
        if self.index is None:
            self.create_index()
        
        start_id = len(self.file_paths)
        
        # usearch 批量添加
        for i, (vector, path, file_hash) in enumerate(zip(vectors, file_paths, file_hashes)):
            idx = start_id + i
            self.index.add(idx, vector)
            
            self.file_paths.append(path)
            self.file_metadata[file_hash] = {
                "path": path,
                "indexed_at": datetime.now().isoformat(),
                "index_id": idx
            }
        
        logger.info(f"Added {len(vectors)} vectors to usearch index")
    
    def search(self, query_vector: np.ndarray, top_k: int = 10) -> List[Tuple[int, float]]:
        """搜索最相似的向量"""
        if self.index is None:
            return []
        
        # usearch 搜索
        # 返回 Match 对象列表，每个对象有 key 和 distance 属性
        matches = self.index.search(query_vector, count=top_k)
        
        # 转换为列表
        search_results = []
        for match in matches:
            # match 是 Match 对象，有 .key 和 .distance 属性
            idx = int(match.key)
            similarity = max(0.0, 1.0 - float(match.distance))
            if idx >= 0:
                search_results.append((idx, similarity))
        
        return search_results
    
    def save(self):
        """保存索引到磁盘"""
        if self.index is None:
            logger.warning("No index to save")
            return
        
        # 保存 usearch 索引
        index_path = self.index_dir / "usearch.index"
        self.index.save(str(index_path))
        
        # 保存元数据
        metadata_path = self.index_dir / "metadata.pkl"
        with open(metadata_path, 'wb') as f:
            pickle.dump({
                "file_metadata": self.file_metadata,
                "file_paths": self.file_paths,
                "dimension": self.dimension,
                "backend": "usearch"
            }, f)
        
        logger.info(f"usearch index saved to {self.index_dir}")
    
    def load(self):
        """从磁盘加载索引"""
        index_path = self.index_dir / "usearch.index"
        metadata_path = self.index_dir / "metadata.pkl"
        
        if not index_path.exists():
            # 尝试加载旧的 faiss 索引
            faiss_index_path = self.index_dir / "faiss.index"
            if faiss_index_path.exists():
                logger.warning("Found FAISS index but usearch not available. Please install usearch.")
            logger.info("No existing usearch index found, will create new one")
            self.create_index()
            return
        
        # 加载 usearch 索引
        self.index = ui.Index.restore(str(index_path))
        
        # 加载元数据
        with open(metadata_path, 'rb') as f:
            metadata = pickle.load(f)
            self.file_metadata = metadata["file_metadata"]
            self.file_paths = metadata["file_paths"]
            self.dimension = metadata.get("dimension", self.dimension)
        
        self.loaded = True
        logger.info(f"usearch index loaded from {self.index_dir}")
    
    def get_stats(self) -> Dict[str, Any]:
        """获取索引统计信息"""
        if self.index is None:
            return {"indexed_files": 0}
        
        return {
            "indexed_files": len(self.file_paths),
            "dimension": self.dimension,
            "index_type": "usearch.HNSW",
            "backend": "usearch",
            "size": self.index.size if hasattr(self.index, 'size') else len(self.file_paths)
        }

    def reset(self):
        """重置索引和元数据"""
        self.index = None
        self.file_metadata = {}
        self.file_paths = []
        self.loaded = False

        for file_name in ("usearch.index", "metadata.pkl", "faiss.index"):
            file_path = self.index_dir / file_name
            if file_path.exists():
                file_path.unlink()

        self.create_index()


class IndexerWorker:
    """索引工作器 - usearch 版本"""
    
    def __init__(self, index_dir: str = "index"):
        self.file_index = FileIndex(index_dir)
        self.parser_worker = None
        self.embedding_worker = None
    
    def index_file(self, file_path: str, content: str, embedding: List[float]) -> str:
        """索引单个文件"""
        # 计算文件哈希
        file_hash = hashlib.md5(content.encode()).hexdigest()
        
        # 添加到索引
        vector = np.array(embedding, dtype=np.float32)
        self.file_index.add_vectors(np.array([vector]), [file_path], [file_hash])
        
        return file_hash
    
    def index_files(self, files: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        批量索引文件
        
        Args:
            files: [{"path": str, "content": str, "embedding": List[float]}, ...]
            
        Returns:
            {"indexed": int, "failed": int, "errors": [...]}
        """
        if not files:
            return {"indexed": 0, "failed": 0}
        
        vectors = []
        paths = []
        hashes = []
        errors = []
        
        for file_info in files:
            try:
                path = file_info["path"]
                content = file_info["content"]
                embedding = file_info["embedding"]
                
                file_hash = hashlib.md5(content.encode()).hexdigest()
                
                vectors.append(embedding)
                paths.append(path)
                hashes.append(file_hash)
                
            except Exception as e:
                errors.append({"path": file_info.get("path", "unknown"), "error": str(e)})
        
        # 批量添加到索引
        if vectors:
            vector_array = np.array(vectors, dtype=np.float32)
            self.file_index.add_vectors(vector_array, paths, hashes)
        
        return {
            "indexed": len(vectors),
            "failed": len(errors),
            "errors": errors
        }
    
    def search(self, query_embedding: List[float], top_k: int = 10) -> List[Dict[str, Any]]:
        """
        搜索相似文件
        
        Args:
            query_embedding: 查询向量
            top_k: 返回数量
            
        Returns:
            [{"path": str, "score": float, "index_id": int}, ...]
        """
        query_vector = np.array(query_embedding, dtype=np.float32)
        results = self.file_index.search(query_vector, top_k)
        
        # 转换为字典列表
        search_results = []
        for idx, score in results:
            if idx < len(self.file_index.file_paths):
                path = self.file_index.file_paths[idx]
                search_results.append({
                    "path": path,
                    "score": score,
                    "index_id": idx
                })
        
        return search_results
    
    def save(self):
        """保存索引"""
        self.file_index.save()
    
    def load(self):
        """加载索引"""
        self.file_index.load()
    
    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        return self.file_index.get_stats()

    def reset(self):
        """重置索引"""
        self.file_index.reset()


def process_request(request: Dict[str, Any]) -> Dict[str, Any]:
    """处理请求"""
    action = request.get("action")
    
    if action == "init":
        index_dir = request.get("index_dir", "index")
        worker = IndexerWorker(index_dir)
        worker.load()
        return {"status": "initialized", "stats": worker.get_stats()}
    
    elif action == "index":
        files = request.get("files", [])
        worker = IndexerWorker(request.get("index_dir", "index"))
        worker.load()
        result = worker.index_files(files)
        worker.save()
        return result
    
    elif action == "search":
        query_embedding = request.get("query_embedding")
        top_k = request.get("top_k", 10)
        
        if not query_embedding:
            return {"error": "query_embedding is required"}
        
        worker = IndexerWorker(request.get("index_dir", "index"))
        worker.load()
        results = worker.search(query_embedding, top_k)
        return {"results": results, "count": len(results)}
    
    elif action == "stats":
        worker = IndexerWorker(request.get("index_dir", "index"))
        worker.load()
        return worker.get_stats()
    
    elif action == "test":
        # 测试模式
        worker = IndexerWorker(request.get("index_dir", "index"))
        worker.load()
        return {
            "status": "success",
            "stats": worker.get_stats()
        }
    
    else:
        return {"error": f"Unknown action: {action}"}


def main():
    """主函数"""
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        # 命令行测试模式
        request = {"action": "test"}
        result = process_request(request)
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return
    
    # 守护进程模式
    logger.info("Indexer worker (usearch) started")
    
    worker = None
    
    for line in sys.stdin:
        try:
            request = json.loads(line.strip())
            action = request.get("action")
            
            if action == "init":
                index_dir = request.get("index_dir", "index")
                worker = IndexerWorker(index_dir)
                worker.load()
                print(json.dumps({"status": "initialized", "stats": worker.get_stats()}))
            
            elif action == "index":
                if worker is None:
                    print(json.dumps({"error": "Worker not initialized"}))
                    continue
                
                files = request.get("files", [])
                result = worker.index_files(files)
                worker.save()
                print(json.dumps(result))
            
            elif action == "search":
                if worker is None:
                    print(json.dumps({"error": "Worker not initialized"}))
                    continue
                
                query_embedding = request.get("query_embedding")
                top_k = request.get("top_k", 10)
                
                if not query_embedding:
                    print(json.dumps({"error": "query_embedding is required"}))
                    continue
                
                results = worker.search(query_embedding, top_k)
                print(json.dumps({"results": results, "count": len(results)}))
            
            elif action == "stats":
                if worker is None:
                    print(json.dumps({"error": "Worker not initialized"}))
                    continue
                
                print(json.dumps(worker.get_stats()))
            
            elif action == "save":
                if worker is None:
                    print(json.dumps({"error": "Worker not initialized"}))
                    continue
                
                worker.save()
                print(json.dumps({"status": "saved"}))

            elif action == "reset":
                if worker is None:
                    print(json.dumps({"error": "Worker not initialized"}))
                    continue

                worker.reset()
                print(json.dumps({"status": "reset", "stats": worker.get_stats()}))
            
            elif action == "quit":
                logger.info("Shutting down")
                if worker:
                    worker.save()
                print(json.dumps({"status": "shutdown"}))
                break
            
            else:
                print(json.dumps({"error": f"Unknown action: {action}"}))
            
            sys.stdout.flush()
            
        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON: {e}")
            print(json.dumps({"error": f"Invalid JSON: {e}"}))
            sys.stdout.flush()
        except Exception as e:
            logger.error(f"Error processing request: {e}")
            print(json.dumps({"error": str(e)}))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
