# === Stage 1: 构建前端 (React) ===
FROM node:24-alpine AS frontend
WORKDIR /app
COPY ui/package*.json ./
# 安装依赖
RUN npm install
# 拷贝源码并打包
COPY ui/ ./
RUN npm run build
# 产物现在位于 /app/dist

# === Stage 2: 构建后端 (Rust) ===
FROM rust:1.93-slim-bookworm AS backend
WORKDIR /app
# 安装构建所需的依赖 (SQLite 需要 C 语言库)
RUN apt-get update && apt-get install -y pkg-config libssl-dev libc6-dev libsqlite3-dev
# 拷贝后端源码
COPY . .
# 编译 Release 版本 (这步比较慢，第一次可能需要几分钟)
RUN cargo build --release

# === Stage 3: 最终运行时 (Runtime) ===
FROM debian:bookworm-slim
WORKDIR /app

# 安装运行时依赖 (SQLite 动态库, SSL)
RUN apt-get update && apt-get install -y libssl3 libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*

# 1. 从 Stage 2 拷贝编译好的 Rust 二进制文件
COPY --from=backend /app/target/release/memeDB ./server
# 2. 从 Stage 1 拷贝打包好的 React 静态文件 (注意路径要和 main.rs 里一致)
COPY --from=frontend /app/dist ./ui/dist

# 创建数据挂载点
RUN mkdir -p storage

# 启动！
CMD ["./server"]
