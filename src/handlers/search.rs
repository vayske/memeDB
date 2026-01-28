use crate::models::Image;
use crate::services;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::sqlite::SqlitePool; // 引入 QueryBuilder

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub tags: Option<String>,
}

pub async fn search_images(
    State(pool): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. 解析标签参数: "cat,funny" -> ["cat", "funny"]
    let tags: Vec<String> = params
        .tags
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let images = services::find_images_by_tags(&pool, tags)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(format_response(images))
}

// 辅助函数：统一返回格式
fn format_response(images: Vec<Image>) -> Json<serde_json::Value> {
    let results: Vec<_> = images
        .into_iter()
        .map(|img| {
            json!({
                "id": img.id,
                "url": format!("/images/{}.{}", img.filename, img.ext),
                "filename": img.filename,
                "ext": img.ext
            })
        })
        .collect();

    Json(json!({
        "code": 200,
        "msg": "Success",
        "data": results
    }))
}
