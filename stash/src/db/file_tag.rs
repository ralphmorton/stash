use sqlx::{Executor, Sqlite, prelude::FromRow, query, query_as};

#[derive(Debug, FromRow)]
pub struct FileTag {
    pub id: i64,
    pub file_id: i64,
    pub tag_id: i64,
}

impl FileTag {
    pub async fn for_file<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        file_id: i64,
    ) -> Result<Vec<FileTag>, sqlx::Error> {
        query_as::<_, FileTag>("SELECT * FROM file_tags WHERE file_id = $1")
            .bind(file_id)
            .fetch_all(conn)
            .await
    }

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

    pub async fn delete<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        id: i64,
    ) -> Result<u64, sqlx::Error> {
        query("DELETE FROM file_tags WHERE id = $1")
            .bind(id)
            .execute(conn)
            .await
            .map(|r| r.rows_affected())
    }
}
