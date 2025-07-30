use chrono::NaiveDateTime;
use sqlx::{Executor, Sqlite, prelude::FromRow, query_as};

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created: NaiveDateTime,
}

impl Tag {
    pub async fn all<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        query_as::<_, Tag>("SELECT * FROM tags ORDER BY name")
            .fetch_all(conn)
            .await
    }

    pub async fn by_name<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        name: &str,
    ) -> Result<Option<Tag>, sqlx::Error> {
        query_as::<_, Tag>("SELECT * FROM tags WHERE name = $1")
            .bind(name)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        name: &str,
    ) -> Result<Tag, sqlx::Error> {
        query_as::<_, Tag>(
            "INSERT INTO tags (name, created) VALUES ($1, datetime('now')) RETURNING *",
        )
        .bind(name)
        .fetch_one(conn)
        .await
    }
}
