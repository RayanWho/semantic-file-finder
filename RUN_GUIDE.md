# 🚀 Semantic File Finder - 快速运行指南

## ⚡ 一键启动（推荐）

### 开发模式

```bash
cd /Users/who/Desktop/semantic-file-finder
./start-dev.sh
```

### 生产构建

```bash
cd /Users/who/Desktop/semantic-file-finder
./build.sh
```

---

## 📋 手动启动步骤

### 方式 1: 开发模式（实时热更新）

```bash
# 1. 进入项目目录
cd /Users/who/Desktop/semantic-file-finder

# 2. 安装依赖（首次运行）
npm install

# 3. 安装 Python 依赖
cd python-workers
pip3 install -r requirements.txt
cd ..

# 4. 启动开发环境
npm run tauri dev
```

**启动后：**
- 🌐 前端：http://localhost:5173
- 🖥️ Tauri 应用：自动打开
- 🔥 热更新：代码修改自动刷新

---

### 方式 2: 生产模式（构建可执行文件）

```bash
# 1. 进入项目目录
cd /Users/who/Desktop/semantic-file-finder

# 2. 构建应用
npm run tauri build
```

**构建产物：**
```
src-tauri/target/release/bundle/
├── dmg/              # macOS 安装包
├── msi/              # Windows 安装包
└── deb/              # Linux 安装包
```

---

## 🔧 组件说明

### 前端 (React + Vite)
```bash
# 单独启动前端开发服务器
npm run dev
```

### 后端 (Tauri + Rust)
```bash
# 单独启动 Tauri 开发
cd src-tauri
cargo tauri dev
```

### Python Workers
```bash
# 测试 embedding worker
cd python-workers
python embedding_worker.py --test

# 测试 indexer worker (usearch 版本)
python indexer_worker_usearch.py --test
```

---

## 📦 依赖检查清单

### 必需
- [x] Node.js ≥ 18
- [x] Rust ≥ 1.70
- [x] Python ≥ 3.9

### Python 依赖
- [x] usearch (向量索引)
- [x] bge-m3 (嵌入模型)
- [x] onnxruntime (模型推理)
- [x] transformers
- [x] torch

### 模型文件
- [ ] `models/onnx/model.onnx` (bge-m3 模型)

**下载模型：**
```bash
mkdir -p models
huggingface-cli download BAAI/bge-m3 onnx/model.onnx --local-dir models
```

---

## 🐛 常见问题

### Q1: npm install 失败
```bash
# 清理缓存
npm cache clean --force
npm install
```

### Q2: Python 依赖安装失败
```bash
# 升级 pip
pip3 install --upgrade pip

# 使用国内镜像
pip3 install -r requirements.txt -i https://pypi.tuna.tsinghua.edu.cn/simple
```

### Q3: 模型下载太慢
```bash
# 使用国内镜像
export HF_ENDPOINT=https://hf-mirror.com
huggingface-cli download BAAI/bge-m3 onnx/model.onnx --local-dir models
```

### Q4: Tauri 构建失败
```bash
# 清理构建缓存
cd src-tauri
cargo clean
cargo tauri build
```

### Q5: usearch 未安装
```bash
cd python-workers
pip3 install usearch
```

---

## 📊 项目结构

```
semantic-file-finder/
├── src/                    # React 前端
│   ├── App.tsx
│   └── components/
├── src-tauri/              # Tauri 后端 (Rust)
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── python-workers/         # Python 工作器
│   ├── indexer_worker_usearch.py  # ✅ 索引 (usearch)
│   ├── embedding_worker.py        # 嵌入
│   └── parser_worker.py           # 解析
├── models/                 # AI 模型
│   └── onnx/model.onnx
├── start-dev.sh            # 🚀 开发启动脚本
├── build.sh                # 🏗️ 生产构建脚本
└── package.json
```

---

## 🎯 快速验证

```bash
# 1. 检查 Node.js
node --version  # 应显示 v18+

# 检查 Rust
rustc --version  # 应显示 1.70+

# 检查 Python
python3 --version  # 应显示 3.9+

# 2. 测试 Python Workers
cd python-workers
python indexer_worker_usearch.py --test  # 应显示成功

# 3. 启动开发环境
cd ..
npm run tauri dev  # 应自动打开应用窗口
```

---

## 📚 下一步

1. **配置模型** - 下载 bge-m3 模型
2. **启动应用** - `./start-dev.sh`
3. **测试搜索** - 在应用中搜索文件
4. **开发功能** - 编辑 `src/` 和 `src-tauri/src/`

---

**开始使用吧！** 💻

有任何问题随时询问！
