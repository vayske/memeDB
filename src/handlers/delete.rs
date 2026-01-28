use axum::{
    extract::{Path, State},
    response::{Json, IntoResponse},
    http::StatusCode,
};
use sqlx::sqlite::SqlitePool;
use serde_json::json;

use crate::services;

pub async fn delete_image(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    // 调用 Service
    services::delete_image(&pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "code": 200,
        "msg": "Deleted successfully",
        "data": null
    })))
}
