use chrono::NaiveDateTime;
use sqlx::{Executor, Sqlite, prelude::FromRow, query, query_as};

use crate::SHA256;

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct File {
    pub id: i64,
    pub name: String,
    pub content_id: i64,
    pub uploader: String,
    pub created: NaiveDateTime,
}

#[derive(Debug, FromRow)]
pub struct FileDesc {
    pub name: String,
    pub size: i64,
    pub hash: SHA256,
    pub created: NaiveDateTime,
}

impl File {
    pub async fn by_name<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        name: &str,
    ) -> Result<Option<File>, sqlx::Error> {
        query_as::<_, File>("SELECT * FROM files WHERE name = $1")
            .bind(name)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        name: &str,
        content_id: i64,
        uploader: &str,
    ) -> Result<File, sqlx::Error> {
        query_as::<_, File>(
            "INSERT INTO files (name, content_id, uploader, created) VALUES ($1, $2, $3, datetime('now')) RETURNING *",
        )
        .bind(name)
        .bind(content_id)
        .bind(uploader)
        .fetch_one(conn)
        .await
    }

    pub async fn delete<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        id: i64,
    ) -> Result<u64, sqlx::Error> {
        query("DELETE FROM files WHERE id = $1")
            .bind(id)
            .execute(conn)
            .await
            .map(|r| r.rows_affected())
    }

    pub async fn search<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        tag: &str,
        term: &str,
    ) -> Result<Vec<FileDesc>, sqlx::Error> {
        query_as::<_, FileDesc>(
            r#"
                SELECT f.name, c.size, c.hash, f.created
                FROM file_tags ft
                JOIN tags t ON t.id = ft.tag_id
                JOIN files f ON f.id = ft.file_id
                JOIN file_contents c ON c.id = f.content_id
                WHERE t.name = $1 AND f.name LIKE $2
                ORDER BY f.name
            "#,
        )
        .bind(tag)
        .bind(term)
        .fetch_all(conn)
        .await
    }
}
