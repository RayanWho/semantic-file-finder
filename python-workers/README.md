# Python Workers for Semantic File Finder

## 安装依赖

```bash
pip install -r requirements.txt
```

## Worker 列表

### embedding_worker.py
bge-m3 嵌入模型，负责文本向量化

### parser_worker.py
文件解析器，支持多种文件格式

### indexer_worker.py
FAISS 索引管理

## 使用方式

```bash
# 测试 embedding
python embedding_worker.py --test "这是一段测试文本"

# 测试文件解析
python parser_worker.py --test /path/to/file.txt

# 测试索引
python indexer_worker.py --test
```
