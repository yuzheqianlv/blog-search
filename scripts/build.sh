#!/bin/bash
set -e

# 清理并创建必要的目录
rm -rf .vercel
rm -rf target
rm -rf data

# 创建静态目录
mkdir -p static

# 复制静态文件
cp -f test.html static/index.html
cp -f test.html static/test.html

# 安装依赖
npm install

# 运行索引器
cargo run --bin indexer --release 