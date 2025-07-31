use std::{str::FromStr, sync::Arc};

use iroh::NodeId;
use sqlx::{Sqlite, SqlitePool, Transaction, prelude::FromRow, query, query_as};
use stash::NodeAuth;
use tokio::sync::RwLock;

pub struct Auth {
    db: SqlitePool,
    admin: NodeId,
    allow: AllowList,
}

type AllowList = Arc<RwLock<Vec<NodeId>>>;

#[derive(FromRow)]
struct AllowedNode {
    node: String,
}

impl Auth {
    pub async fn new(db: SqlitePool, admin: NodeId) -> Result<Self, sqlx::Error> {
        let mut transaction = db.begin().await?;
        let allowed = allowed(&mut transaction).await?;
        transaction.commit().await?;

        let i = Self {
            db,
            admin,
            allow: Arc::new(RwLock::new(allowed)),
        };

        Ok(i)
    }
}

impl NodeAuth for Auth {
    async fn allow(&self, node: NodeId) -> bool {
        if node == self.admin {
            return true;
        }

        self.allow.read().await.iter().any(|n| n == &node)
    }

    async fn add(&self, caller: NodeId, node: NodeId) -> bool {
        if caller != self.admin {
            return false;
        }

        if self.allow(node).await {
            return true;
        }

        if let Err(e) = add(&self.db, node, self.allow.clone()).await {
            tracing::error!(err = ?e, "add_node_failed");
        }

        true
    }

    async fn remove(&self, caller: NodeId, node: NodeId) -> bool {
        if caller != self.admin {
            return false;
        }

        if let Err(e) = remove(&self.db, node, self.allow.clone()).await {
            tracing::error!(err = ?e, "remove_node_failed");
        }

        true
    }
}

async fn allowed(
    transaction: &mut Transaction<'static, Sqlite>,
) -> Result<Vec<NodeId>, sqlx::Error> {
    let nodes = query_as::<_, AllowedNode>("SELECT * FROM allowed_nodes")
        .fetch_all(&mut **transaction)
        .await?
        .iter()
        .filter_map(|n| NodeId::from_str(&n.node).ok())
        .collect();

    Ok(nodes)
}

async fn add(db: &SqlitePool, node: NodeId, allow: AllowList) -> Result<(), sqlx::Error> {
    let mut transaction = db.begin().await?;

    query("INSERT INTO allowed_nodes (node) VALUES ($1)")
        .bind(&format!("{node}"))
        .execute(&mut *transaction)
        .await?;

    allow.write().await.push(node);

    transaction.commit().await?;
    Ok(())
}

async fn remove(db: &SqlitePool, node: NodeId, allow: AllowList) -> Result<(), sqlx::Error> {
    let mut transaction = db.begin().await?;

    query("DELETE FROM allowed_nodes WHERE node = $1")
        .bind(&format!("{node}"))
        .execute(&mut *transaction)
        .await?;

    let allowed = allowed(&mut transaction).await?;
    *allow.write().await = allowed;

    transaction.commit().await?;
    Ok(())
}
