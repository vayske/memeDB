use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Image {
    pub id: i64,
    pub filename: String,
    pub ext: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
