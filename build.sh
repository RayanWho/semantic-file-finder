#!/bin/bash
# Semantic File Finder - 生产环境构建脚本

set -e

echo "🏗️  Semantic File Finder - 生产构建"
echo "===================================="

PROJECT_DIR="/Users/who/Desktop/semantic-file-finder"
cd "$PROJECT_DIR"

# 1. 安装依赖
echo ""
echo "📦 步骤 1: 安装依赖..."
npm install

# 2. 构建 Tauri 应用
echo ""
echo "🏗️  步骤 2: 构建应用..."
npm run tauri build

echo ""
echo "✅ 构建完成！"
echo ""
echo "📦 构建产物位置："
echo "   src-tauri/target/release/bundle/"
echo ""
echo "🎉 可以分发给用户了！"
