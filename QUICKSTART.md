# 快速启动指南

## 🚀 5 分钟开始开发

### 1. 检查前置条件

```bash
# 检查 Node.js
node --version  # 需要 ≥ 18

# 检查 Rust
rustc --version  # 需要 ≥ 1.70

# 检查 Python
python --version  # 需要 ≥ 3.9
```

### 2. 安装依赖

```bash
cd /Users/who/Desktop/semantic-file-finder

# 安装前端依赖
npm install

# 安装 Python 依赖
cd python-workers
pip install -r requirements.txt
cd ..
```

### 3. 下载模型

```bash
# 创建模型目录
mkdir -p models

# 下载 bge-m3 模型 (约 1.2GB)
# 方式 1: 使用 huggingface-cli
pip install huggingface-hub
huggingface-cli download BAAI/bge-m3 onnx/model.onnx --local-dir models

# 方式 2: 手动下载
# 访问 https://huggingface.co/BAAI/bge-m3/tree/main/onnx
# 下载 model.onnx 到 models/ 目录
```

### 4. 启动开发环境

```bash
# 方式 1: 使用 npm 脚本
npm run tauri dev

# 方式 2: 直接使用 Tauri CLI
cd src-tauri
cargo tauri dev
```

### 5. 构建发布版本

```bash
# 构建当前平台
npm run tauri build

# 构建特定平台
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS Intel
npm run tauri build -- --target x86_64-apple-darwin

# macOS Apple Silicon
npm run tauri build -- --target aarch64-apple-darwin

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## 📦 构建产物

构建完成后，可执行文件位于：

```
src-tauri/target/release/
├── semantic-file-finder          # macOS/Linux 可执行文件
├── semantic-file-finder.exe      # Windows 可执行文件
└── bundle/
    ├── deb/                      # Linux .deb 包
    ├── dmg/                      # macOS .dmg
    └── msi/                      # Windows .msi
```

## 🐛 常见问题

### Q: Rust 版本太低

```bash
rustup update
```

### Q: Python 依赖安装失败

```bash
# 升级 pip
pip install --upgrade pip

# 使用国内镜像
pip install -r requirements.txt -i https://pypi.tuna.tsinghua.edu.cn/simple
```

### Q: 模型下载太慢

```bash
# 使用国内镜像
export HF_ENDPOINT=https://hf-mirror.com
huggingface-cli download BAAI/bge-m3 onnx/model.onnx --local-dir models
```

### Q: Tauri 构建失败

```bash
# 清理构建缓存
cd src-tauri
cargo clean

# 重新构建
cargo tauri build
```

## 📝 下一步

1. **实现搜索功能** - 编辑 `src-tauri/src/state.rs` 中的 `search()` 方法
2. **实现索引功能** - 编辑 `python-workers/indexer_worker.py`
3. **完善 UI** - 编辑 `src/` 下的 React 组件
4. **添加测试** - 编写单元测试和 E2E 测试

## 📚 参考文档

- [Tauri v2 文档](https://v2.tauri.app/)
- [React 文档](https://react.dev/)
- [bge-m3 模型](https://huggingface.co/BAAI/bge-m3)
- [FAISS 文档](https://faiss.ai/)

---

**开始编码吧！** 💻
