use chrono::NaiveDateTime;
use sqlx::{Executor, Sqlite, prelude::FromRow, query_as};

use crate::SHA256;

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct FileContent {
    pub id: i64,
    pub size: i64,
    pub hash: SHA256,
    pub created: NaiveDateTime,
}

impl FileContent {
    pub async fn by_hash<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        hash: &SHA256,
    ) -> Result<Option<FileContent>, sqlx::Error> {
        query_as::<_, FileContent>("SELECT * FROM file_contents WHERE hash = $1")
            .bind(hash)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        size: i64,
        hash: &SHA256,
    ) -> Result<FileContent, sqlx::Error> {
        query_as::<_, FileContent>(
            "INSERT INTO file_contents (size, hash, created) VALUES ($1, $2, datetime('now')) RETURNING *",
        )
        .bind(size)
        .bind(hash)
        .fetch_one(conn)
        .await
    }
}
