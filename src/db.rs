use sqlx::sqlite::SqlitePool;

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA foreign_keys = ON;").execute(pool).await?;

    let schema_sql = r#"
    CREATE TABLE IF NOT EXISTS images (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        filename    TEXT UNIQUE NOT NULL,
        ext         TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS tags (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        name        TEXT UNIQUE NOT NULL
    );

    CREATE TABLE IF NOT EXISTS image_tags (
        image_id    INTEGER NOT NULL,
        tag_id      INTEGER NOT NULL,
        PRIMARY KEY (image_id, tag_id),
        FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
        FOREIGN KEY (tag_id)   REFERENCES tags(id)   ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_tag_image ON image_tags(tag_id, image_id);
    "#;

    sqlx::query(schema_sql).execute(pool).await?;

    Ok(())
}
