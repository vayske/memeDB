use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::sqlite::SqlitePool;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::models::Image;
use crate::services;

// 定义上传成功返回的 JSON 格式
// 简单的 Result<T, E> 处理：如果成功返回 (200, JSON)，如果失败返回 (500, String)
pub async fn upload_image(
    State(pool): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. 遍历上传的数据流
    // Multipart 可能会包含多个字段，我们需要找到 "file" 字段
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    {
        // 这里的 "file" 必须跟前端表单里的 name="file" 一致
        if field.name() == Some("file") {
            // 获取原始文件名 (例如 "cat.jpg")
            let original_filename = field.file_name().unwrap_or("unknown.bin").to_string();
            // 提取后缀名 (jpg)
            let ext = Path::new(&original_filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin")
                .to_string();

            // 2. 读取文件内容到内存
            // 注意：如果文件巨大 (比如几G)，直接读内存会爆。但表情包一般很小，Bytes 读法最简单。
            let data = field
                .bytes()
                .await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

            // 3. 计算 SHA256 哈希 (指纹)
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let result = hasher.finalize();
            let hash_filename = hex::encode(result); // 变成 "a1b2c3..."

            // 4. 【查重逻辑】先去数据库看看有没有这张图
            // 我们的 SQL 以前叫 filename，现在里面存的是 hash 值
            let existing: Option<Image> = services::find_image_by_hash(&pool, &hash_filename)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            if let Some(img) = existing {
                // A. 如果数据库里有：这是秒传！
                return Ok(Json(json!({
                    "code": 200,
                    "msg": "Image exists (deduplicated)",
                    "data": { "id": img.id, "filename": img.filename, "ext": img.ext }
                })));
            }

            // B. 如果数据库里没有：存硬盘 + 存数据库

            // 5. 保存文件到 ./storage/hash.ext
            let save_path = format!("storage/{}.{}", hash_filename, ext);
            let mut file = File::create(&save_path)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            file.write_all(&data)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // 6. 写入数据库
            let id = services::create_image(&pool, &hash_filename, &ext)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            return Ok(Json(json!({
                "code": 200,
                "msg": "Upload success",
                "data": { "id": id, "filename": hash_filename, "ext": ext }
            })));
        }
    }

    // 如果循环结束了都没找到 file 字段
    Err((StatusCode::BAD_REQUEST, "Missing 'file' field".to_string()))
}
