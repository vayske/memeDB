use sqlx::{sqlite::SqlitePool, QueryBuilder, Row};
use crate::models::{Image, Tag};

pub async fn find_images_by_tags(pool: &SqlitePool, tags: Vec<String>) -> Result<Vec<Image>, sqlx::Error> {

    // 1. 如果没有标签，查最新的 50 张
    if tags.is_empty() {
        let images = sqlx::query_as::<_, Image>("SELECT * FROM images ORDER BY id DESC LIMIT 50")
            .fetch_all(pool)
            .await?;
        return Ok(images);
    }

    // 2. 如果有标签，构建复杂 SQL (逻辑和你之前写的一样)
    let mut builder = QueryBuilder::new(r#"
        SELECT images.id, images.filename, images.ext
        FROM images
        INNER JOIN image_tags ON images.id = image_tags.image_id
        INNER JOIN tags ON image_tags.tag_id = tags.id
        WHERE tags.name IN (
    "#);

    let mut separated = builder.separated(", ");
    for tag in &tags {
        separated.push_bind(tag);
    }
    separated.push_unseparated(") ");

    builder.push("GROUP BY images.id HAVING COUNT(DISTINCT tags.id) = ");
    builder.push_bind(tags.len() as i64);
    builder.push(" ORDER BY images.id DESC");

    let query = builder.build();

    let images = query
        .fetch_all(pool)
        .await?
        .iter()
        .map(|row| Image {
            id: row.get("id"),
            filename: row.get("filename"),
            ext: row.get("ext"),
        })
        .collect();

    Ok(images)
}

pub async fn find_image_by_hash(pool: &SqlitePool, hash: &str) -> Result<Option<Image>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM images WHERE filename = ?")
        .bind(hash)
        .fetch_optional(pool)
        .await
}

pub async fn create_image(pool: &SqlitePool, filename: &str, ext: &str) -> Result<i64, sqlx::Error> {
    let id = sqlx::query("INSERT INTO images (filename, ext) VALUES (?, ?)")
        .bind(filename)
        .bind(ext)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn add_tag_to_image(pool: &SqlitePool, image_id: i64, tag_name: &str) -> Result<(), sqlx::Error> {
    // 1. 插入标签 (如果存在则忽略)
    sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?)")
        .bind(tag_name)
        .execute(pool)
        .await?;

    // 2. 获取标签 ID
    let tag_id: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE name = ?")
        .bind(tag_name)
        .fetch_one(pool)
        .await?;

    // 3. 建立关联
    sqlx::query("INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?, ?)")
        .bind(image_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_tags_by_image_id(pool: &SqlitePool, image_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as(r#"
        SELECT tags.id, tags.name
        FROM tags
        INNER JOIN image_tags ON tags.id = image_tags.tag_id
        WHERE image_tags.image_id = ?
    "#)
    .bind(image_id)
    .fetch_all(pool)
    .await
}

pub async fn list_all_tags(pool: &SqlitePool) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM tags ORDER BY name ASC")
        .fetch_all(pool)
        .await
}

pub async fn delete_image(pool: &SqlitePool, id: i64) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 先查出文件信息 (为了最后删硬盘上的文件)
    // 如果查不到，说明 ID 不存在，直接返回错误
    let image = sqlx::query_as::<_, Image>("SELECT * FROM images WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or("Image not found")?;

    // 2. 开启事务 (Transaction)
    // 事务能保证：如果删数据库中途出错，之前的操作会自动回滚，不会留下烂摊子
    let mut tx = pool.begin().await?;

    // 3. 删除 image_tags 表里的关联数据 (必须先做这个！)
    sqlx::query("DELETE FROM image_tags WHERE image_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // 4. 删除 images 表里的主记录
    sqlx::query("DELETE FROM images WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // 5. 提交事务 (这时候数据库操作才算真正生效)
    tx.commit().await?;

    // 6. 最后：删除硬盘上的物理文件
    // 我们使用 std::fs 或 tokio::fs。这里路径要和 upload 里的保持一致。
    let file_path = format!("storage/{}.{}", image.filename, image.ext);
    let path = std::path::Path::new(&file_path);

    if path.exists() {
        // 如果删除文件失败，我们打印个日志，但不要让接口报错
        // 因为数据库已经删干净了，文件删不删只影响磁盘占用，不影响业务
        if let Err(e) = tokio::fs::remove_file(path).await {
            eprintln!("Failed to delete file {}: {}", file_path, e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*; // 引入父模块的所有函数
    use sqlx::sqlite::SqlitePoolOptions;

    // 这是一个异步测试
    #[tokio::test]
    async fn test_create_image_and_add_tag() {
        // 1. 准备一个"内存数据库" (每次跑测试都是全新的，不污染 meme.db)
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:") // <--- 关键！用内存模式
            .await
            .unwrap();

        // 2. 初始化表结构 (我们需要手动跑一下建表 SQL)
        sqlx::query("
            CREATE TABLE images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL UNIQUE,
                ext TEXT NOT NULL
            );
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE image_tags (
                image_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (image_id, tag_id),
                FOREIGN KEY(image_id) REFERENCES images(id),
                FOREIGN KEY(tag_id) REFERENCES tags(id)
            );
        ")
        .execute(&pool)
        .await
        .unwrap();

        // 3. 开始测试业务逻辑
        // A. 创建图片
        let image_id = create_image(&pool, "test_hash_123", "jpg").await.unwrap();
        assert_eq!(image_id, 1); // 应该是第一条数据

        // B. 打标签
        add_tag_to_image(&pool, image_id, "funny").await.unwrap();
        add_tag_to_image(&pool, image_id, "cat").await.unwrap();

        // C. 查标签
        let tags = list_all_tags(&pool).await.unwrap();
        assert_eq!(tags.len(), 2); // 应该有2个标签
        assert!(tags.iter().any(|t| t.name == "funny")); // 验证包含 funny

        // D. 按标签搜图
        let images = find_images_by_tags(&pool, vec!["funny".to_string()]).await.unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].filename, "test_hash_123");

        println!("✅ 测试通过！业务逻辑没问题。");
    }
}
