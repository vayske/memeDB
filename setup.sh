#!/bin/bash

# 遇到错误立即停止
set -e

# 定义一些颜色，让输出好看点
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting MemeDB Setup...${NC}"

# ==========================================
# 1. 目录与权限准备 (Pre-flight Checks)
# ==========================================
echo -e "${GREEN}==> Preparing directories...${NC}"

# 创建 storage 目录 (如果不存在)
if [ ! -d "storage" ]; then
    mkdir -p storage
    echo "    Created 'storage' directory."
else
    echo "    'storage' directory already exists."
fi

# 创建 data 目录
if [ ! -d "data" ]; then
    mkdir -p data
    echo "    Created 'data' directory."
else
    echo "    'data' directory already exists."
fi

# [关键点]：预先创建空的数据库文件
# 如果不这样做，Docker 启动时如果发现挂载的文件不存在，
# 有时候会把它当成一个“文件夹”创建出来，导致程序报错。
# 或者 Rust 程序因为权限不足无法在 data/ 下创建文件。
if [ ! -f "data/meme.db" ]; then
    touch data/meme.db
    echo "    Created empty 'meme.db' file to prevent permission issues."
fi

# ==========================================
# 2. 启动 Docker Compose
# ==========================================
echo -e "${GREEN}==> Building and starting containers...${NC}"

# 检查 docker compose 命令是否存在
if docker compose version >/dev/null 2>&1; then
    CMD="docker compose"
elif docker-compose --version >/dev/null 2>&1; then
    CMD="docker-compose"
else
    echo "Error: 'docker compose' is not installed."
    exit 1
fi

# 运行构建并后台启动
# --build: 确保每次运行脚本都重新编译代码 (防止你改了 Rust 代码却还在跑旧镜像)
$CMD up --build -d

# ==========================================
# 3. 完成提示
# ==========================================
echo -e "${BLUE}=======================================${NC}"
echo -e "${GREEN}✅ MemeDB is running!${NC}"
echo -e "   Frontend & Backend: http://localhost:8081"
echo -e "${BLUE}=======================================${NC}"
echo -e "To view logs, run: ${CMD} logs -f"
echo -e "To stop, run:      ${CMD} down"
