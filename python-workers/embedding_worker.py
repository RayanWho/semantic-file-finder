#!/usr/bin/env python3
"""
Embedding Worker - bge-m3 文本向量化
使用 ONNX Runtime 加速推理
"""

import sys
import json
import logging
import numpy as np
from typing import List, Dict, Any
from pathlib import Path

try:
    import onnxruntime as ort
    from transformers import AutoTokenizer
    import torch
except ImportError as e:
    print(json.dumps({"error": f"Missing dependency: {e}"}))
    sys.exit(1)

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("embedding_worker")


class EmbeddingModel:
    """bge-m3 嵌入模型封装"""
    
    def __init__(self, model_path: str = "models"):
        self.model_path = Path(model_path)
        self.tokenizer = None
        self.session = None
        self.max_length = 8192
        self.loaded = False
        
    def load(self):
        """加载模型"""
        try:
            logger.info(f"Loading model from {self.model_path}")
            
            # 加载 tokenizer
            self.tokenizer = AutoTokenizer.from_pretrained(self.model_path)
            
            # 加载 ONNX 模型
            onnx_path = self.model_path / "onnx" / "model.onnx"
            if not onnx_path.exists():
                # 尝试其他路径
                onnx_path = self.model_path / "model.onnx"
            
            if not onnx_path.exists():
                raise FileNotFoundError(f"ONNX model not found at {onnx_path}")
            
            # 使用 CPU 推理
            providers = ['CPUExecutionProvider']
            self.session = ort.InferenceSession(str(onnx_path), providers=providers)
            
            self.loaded = True
            logger.info("Model loaded successfully")
            
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            raise
    
    def encode(self, texts: List[str], batch_size: int = 16) -> List[List[float]]:
        """
        将文本列表转换为向量
        
        Args:
            texts: 文本列表
            batch_size: 批处理大小
            
        Returns:
            向量列表
        """
        if not self.loaded:
            self.load()
        
        all_embeddings = []
        
        for i in range(0, len(texts), batch_size):
            batch_texts = texts[i:i + batch_size]
            
            # Tokenize
            inputs = self.tokenizer(
                batch_texts,
                padding=True,
                truncation=True,
                max_length=self.max_length,
                return_tensors="pt"
            )
            
            # 准备输入
            input_ids = inputs["input_ids"].numpy()
            attention_mask = inputs["attention_mask"].numpy()
            
            # 推理
            ort_inputs = {
                "input_ids": input_ids,
                "attention_mask": attention_mask
            }
            
            # 如果有 token_type_ids
            if "token_type_ids" in inputs:
                ort_inputs["token_type_ids"] = inputs["token_type_ids"].numpy()
            
            # 运行推理
            outputs = self.session.run(None, ort_inputs)
            
            # 获取 embeddings (通常是最后一个输出)
            # bge-m3 输出格式：[batch_size, seq_len, hidden_size]
            # 我们需要 pool 成 [batch_size, hidden_size]
            embeddings = outputs[-1]  # 或 outputs[0]，根据模型输出调整
            
            # Mean pooling
            embeddings = self._mean_pooling(embeddings, attention_mask)
            
            # 归一化
            embeddings = self._normalize(embeddings)
            
            all_embeddings.extend(embeddings.tolist())
        
        return all_embeddings
    
    def _mean_pooling(self, embeddings: np.ndarray, attention_mask: np.ndarray) -> np.ndarray:
        """Mean pooling"""
        input_mask_expanded = np.expand_dims(attention_mask, axis=-1).astype(float)
        sum_embeddings = np.sum(embeddings * input_mask_expanded, axis=1)
        sum_mask = np.clip(input_mask_expanded.sum(axis=1), 1e-9, None)
        return sum_embeddings / sum_mask
    
    def _normalize(self, embeddings: np.ndarray) -> np.ndarray:
        """L2 归一化"""
        norm = np.linalg.norm(embeddings, ord=2, axis=1, keepdims=True)
        return embeddings / norm
    
    def encode_query(self, query: str) -> List[float]:
        """编码查询文本（添加前缀）"""
        # bge-m3 查询前缀
        prefixed_query = "Represent this sentence for searching relevant passages: " + query
        embeddings = self.encode([prefixed_query])
        return embeddings[0]
    
    def encode_documents(self, documents: List[str]) -> List[List[float]]:
        """编码文档列表"""
        return self.encode(documents)


def process_request(request: Dict[str, Any]) -> Dict[str, Any]:
    """处理请求"""
    action = request.get("action")
    
    if action == "load":
        model = EmbeddingModel(request.get("model_path", "models"))
        model.load()
        return {"status": "loaded"}
    
    elif action == "encode":
        model = EmbeddingModel(request.get("model_path", "models"))
        if not model.loaded:
            model.load()
        
        texts = request.get("texts", [])
        is_query = request.get("is_query", False)
        
        if is_query:
            embeddings = [model.encode_query(text) for text in texts]
        else:
            embeddings = model.encode_documents(texts)
        
        return {
            "status": "success",
            "embeddings": embeddings,
            "count": len(embeddings)
        }
    
    elif action == "test":
        # 测试模式
        model = EmbeddingModel(request.get("model_path", "models"))
        model.load()
        
        test_text = request.get("text", "这是一段测试文本")
        embedding = model.encode_query(test_text)
        
        return {
            "status": "success",
            "test_text": test_text,
            "embedding_dim": len(embedding),
            "embedding_preview": embedding[:10]  # 前 10 个维度
        }
    
    else:
        return {"error": f"Unknown action: {action}"}


def main():
    """主函数 - 从 stdin 读取 JSON 请求"""
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        # 命令行测试模式
        test_text = sys.argv[2] if len(sys.argv) > 2 else "测试文本"
        request = {"action": "test", "text": test_text}
        result = process_request(request)
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return
    
    # 守护进程模式 - 从 stdin 读取请求
    logger.info("Embedding worker started")
    
    model = None
    
    for line in sys.stdin:
        try:
            request = json.loads(line.strip())
            action = request.get("action")
            
            if action == "init":
                model = EmbeddingModel(request.get("model_path", "models"))
                model.load()
                print(json.dumps({"status": "initialized"}))
            
            elif action == "encode":
                if model is None:
                    print(json.dumps({"error": "Model not initialized"}))
                    continue
                
                texts = request.get("texts", [])
                is_query = request.get("is_query", False)
                
                if is_query:
                    embeddings = [model.encode_query(text) for text in texts]
                else:
                    embeddings = model.encode_documents(texts)
                
                print(json.dumps({
                    "status": "success",
                    "embeddings": embeddings,
                    "count": len(embeddings)
                }))
            
            elif action == "quit":
                logger.info("Shutting down")
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
