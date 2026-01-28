use crate::models::Tag;
use crate::services;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::sqlite::SqlitePool;

// 前端发过来的 JSON 应该是: { "tags": ["cat", "funny", "meme"] }
#[derive(Debug, Deserialize)]
pub struct AddTagsRequest {
    pub tags: Vec<String>,
}

pub async fn add_tags(
    State(pool): State<SqlitePool>,
    Path(image_id): Path<i64>,
    Json(payload): Json<AddTagsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 我们用一个列表来收集成功打上的标签，最后返回给前端看
    let mut added_tags = Vec::new();

    for raw_tag in payload.tags {
        let tag_name = raw_tag.trim().to_string();

        if tag_name.is_empty() {
            continue;
        }

        services::add_tag_to_image(&pool, image_id, &tag_name)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        added_tags.push(tag_name);
    }

    Ok(Json(json!({
        "code": 200,
        "msg": "Tags added",
        "data": {
            "image_id": image_id,
            "tags": added_tags // 告诉前端实际上打上了哪些标签
        }
    })))
}

pub async fn get_image_tags(
    State(pool): State<SqlitePool>,
    Path(image_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    let tags = services::get_tags_by_image_id(&pool, image_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "code": 200,
        "msg": "Success",
        "data": tags
    })))
}

pub async fn list_tags(
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let tags: Vec<Tag> = services::list_all_tags(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "code": 200,
        "msg": "Success",
        "data": tags
    })))
}
