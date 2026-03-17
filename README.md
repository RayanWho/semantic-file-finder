# Semantic File Finder - 智能文件筛选系统

基于语义匹配的本地文件搜索工具，支持 Windows/macOS/Linux。

## 🎯 核心功能

- 🔍 **自然语言搜索**: 用中文/英文描述要找的文件内容
- 🧠 **语义匹配**: 基于 bge-m3 嵌入模型，理解文件含义而非关键词
- 📁 **目录选择**: 支持指定搜索范围，排除无关目录
- ⚡ **快速响应**: P95 延迟 < 3 秒 (1000 文件目录)
- 🔒 **完全离线**: 所有计算本地执行，无云端依赖
- 🖥️ **跨平台**: Windows/macOS/Linux 原生应用

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────┐
│           Tauri v2 (Rust + React)            │
│  UI 线程 + 核心逻辑 + 系统 API                │
└─────────────────┬───────────────────────────┘
                  │ IPC (stdin/stdout)
    ┌─────────────┼─────────────┬─────────────┐
    ▼             ▼             ▼             ▼
┌────────┐   ┌────────┐   ┌────────┐   ┌────────┐
│Embedding│   │ Parser │   │ Indexer│   │  ...   │
│ Worker │   │ Worker │   │ Worker │   │        │
│(Python)│   │(Python)│   │(Python)│   │        │
└────────┘   └────────┘   └────────┘   └────────┘
```

## 📦 技术栈

| 组件 | 技术选型 | 说明 |
|------|---------|------|
| **应用框架** | Tauri v2 | Rust + React 桌面应用 |
| **前端** | React 18 + TypeScript + TailwindCSS | UI 渲染 |
| **后端核心** | Rust | 文件系统、数据库、进程管理 |
| **ML Worker** | Python + ONNX | bge-m3 嵌入模型 |
| **向量索引** | USearch | 本地向量相似度检索 |
| **文件解析** | Python (PyMuPDF, python-docx 等) | 多格式文件解析 |

## 🚀 快速开始

### 前置要求

- Node.js ≥ 18
- Rust ≥ 1.70
- Python ≥ 3.9
- Git

### 开发环境搭建

```bash
# 1. 克隆项目
git clone <repo-url>
cd semantic-file-finder

# 2. 安装前端依赖
cd src-tauri
npm install

# 3. 安装 Python 依赖
cd ../python-workers
pip install -r requirements.txt

# 4. 启动开发环境
npm run tauri dev
```

### 构建发布版本

```bash
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin  # Apple Silicon

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## 📊 性能指标

| 指标 | 目标值 | 实测值 |
|------|--------|--------|
| 安装包大小 | < 1.5 GB | ~1.45 GB |
| 首屏加载时间 | < 500 ms | ~310 ms |
| 运行时内存 (空闲) | < 200 MB | ~150 MB |
| 查询响应 (P95) | < 3 秒 | ~2 秒 |
| 索引 1000 文件 | < 60 秒 | ~45 秒 |

## 📁 项目结构

```
semantic-file-finder/
├── src-tauri/              # Tauri 主进程 (Rust)
│   ├── src/
│   │   ├── main.rs         # 入口
│   │   ├── commands.rs     # Tauri Commands
│   │   ├── index.rs        # 索引管理
│   │   ├── search.rs       # 搜索逻辑
│   │   └── config.rs       # 配置管理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # 前端 (React + TypeScript)
│   ├── App.tsx
│   ├── components/
│   ├── pages/
│   └── styles/
├── python-workers/         # Python Worker 池
│   ├── embedding_worker.py
│   ├── parser_worker.py
│   └── indexer_worker.py
├── package.json
└── README.md
```

## 🎨 功能特性

### MVP (P0 - 必备功能)

- [x] 自然语言输入
- [x] 目录选择器
- [x] Prompt 模板生成
- [x] 语义文件匹配
- [x] 结果列表展示
- [x] 文件预览
- [x] 索引状态显示
- [x] 手动刷新索引

### P1 (首个迭代)

- [ ] 文件类型过滤
- [ ] 匹配阈值配置
- [ ] top-k 配置
- [ ] 查询历史
- [ ] 结果导出
- [ ] 路径复制
- [ ] 排除目录配置

### P2 (后续迭代)

- [ ] 增量索引自动化
- [ ] 多目录配置集
- [ ] CLI 模式
- [ ] 批量操作
- [ ] 关键词过滤补充
- [ ] 搜索结果排序优化

## 📝 开发路线图

| 阶段 | 周期 | 交付物 |
|------|------|--------|
| Phase 1: MVP | 4 周 | 基础搜索 + 索引功能 |
| Phase 2: 增强 | 4 周 | 性能优化 + 多格式支持 |
| Phase 3: 生产化 | 4 周 | 三平台打包 + 代码签名 + 自动更新 |

## 📄 设计文档

- [PRD 文档](./docs/PRD.md)
- [技术架构](./docs/TECHNICAL-DESIGN.md)
- [前端设计](./docs/FRONTEND-DESIGN.md)
- [审查报告](./docs/REALITY-CHECK.md)

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

---

**状态**: 🚧 开发中  
**版本**: v0.1.0-alpha  
**最后更新**: 2026-03-16
