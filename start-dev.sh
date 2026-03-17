#!/bin/bash
# Semantic File Finder - 快速启动脚本
# 一键启动整个项目（开发模式）

set -e

echo "🚀 Semantic File Finder - 快速启动"
echo "=================================="

PROJECT_DIR="/Users/who/Desktop/semantic-file-finder"
cd "$PROJECT_DIR"

# 1. 检查依赖
echo ""
echo "📦 步骤 1: 检查依赖..."

if ! command -v node &> /dev/null; then
    echo "❌ Node.js 未安装，请先安装 Node.js"
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 未安装，请先安装 Python"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo 未安装，请先安装 Rust"
    exit 1
fi

echo "✅ Node.js: $(node --version)"
echo "✅ Python: $(python3 --version)"
echo "✅ Cargo: $(cargo --version)"

# 2. 安装前端依赖
echo ""
echo "📦 步骤 2: 安装前端依赖..."
if [ ! -d "node_modules" ]; then
    npm install
else
    echo "⚠️  node_modules 已存在，跳过安装"
fi

# 3. 安装 Python 依赖
echo ""
echo "📦 步骤 3: 安装 Python 依赖..."
cd python-workers
if [ ! -d "../models" ]; then
    echo "⚠️  模型目录不存在，请确认模型已下载"
fi
pip3 install -r requirements.txt -q
cd ..

# 4. 检查模型
echo ""
echo "🤖 步骤 4: 检查模型..."
if [ -d "models" ] && [ -f "models/onnx/model.onnx" ]; then
    echo "✅ bge-m3 模型已就绪"
else
    echo "⚠️  模型未找到，请手动下载："
    echo "   mkdir -p models"
    echo "   huggingface-cli download BAAI/bge-m3 onnx/model.onnx --local-dir models"
fi

# 5. 启动开发环境
echo ""
echo "🚀 步骤 5: 启动开发环境..."
echo ""
echo "💡 提示："
echo "   - 前端将在 http://localhost:5173 运行"
echo "   - Tauri 应用将自动打开"
echo "   - 按 Ctrl+C 停止服务"
echo ""

# 启动 Tauri 开发环境
npm run tauri dev
