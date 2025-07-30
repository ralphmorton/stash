use sqlx::{Executor, Sqlite, prelude::FromRow, query_as};

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct FileTag {
    pub id: i64,
    pub file_id: i64,
    pub tag_id: i64,
}

#[derive(FromRow)]
struct Tag {
    name: String,
}

impl FileTag {
    pub async fn insert<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        file_id: i64,
        tag_id: i64,
    ) -> Result<FileTag, sqlx::Error> {
        query_as::<_, FileTag>(
            "INSERT INTO file_tags (file_id, tag_id) VALUES ($1, $2) RETURNING *",
        )
        .bind(file_id)
        .bind(tag_id)
        .fetch_one(conn)
        .await
    }

    pub async fn for_file<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        file_id: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        let tags = query_as::<_, Tag>(
            r#"
                SELECT t.name
                FROM file_tags ft
                JOIN tags t ON t.id = ft.tag_id
                WHERE ft.file_id = $1
                ORDER BY t.name
            "#,
        )
        .bind(file_id)
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|t| t.name)
        .collect();

        Ok(tags)
    }
}
