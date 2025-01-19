#!/bin/bash
set -e

# 清理并创建必要的目录
rm -rf .vercel
rm -rf target
rm -rf data

# 创建静态目录
mkdir -p static

# 确保 test.html 存在
if [ ! -f "static/test.html" ]; then
    cp test.html static/test.html
fi

# 安装依赖
npm install

# 运行索引器
cargo run --bin indexer --release 