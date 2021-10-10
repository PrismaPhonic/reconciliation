use crate::error::Error;
use crate::models::{Hello, HelloRow, HelloStatus, HelloStatusRow};
use chrono::{Duration, Utc};
use sqlx::{MySql, Pool};

/// The repository abstraction over access to `Hello` objects, as written into the `hello` table.
pub struct Hellos {
    pool: Pool<MySql>,
}

impl Hellos {
    pub fn new(pool: Pool<MySql>) -> Hellos {
        Hellos { pool }
    }
}

impl Hellos {
    pub async fn upsert(&mut self, hello: &Hello) -> Result<u64, Error> {
        let status = if let Some(ref s) = hello.status {
            s
        } else {
            return Ok(hello.id);
        };
        sqlx::query!("INSERT INTO hello_status (hello_id, created_at, updated_at, deleted_at, message) VALUES (?, ?, ?, ?, ?)",
            status.hello_id, status.created_at, status.updated_at, status.deleted_at, status.message)
            .execute(&self.pool)
            .await?;

        Ok(hello.id)
    }

    pub async fn get(&mut self, key: &u64) -> Result<Option<Hello>, Error> {
        let result: Option<HelloRow> = sqlx::query_as!(
            HelloRow,
            "SELECT id, created_at, updated_at, deleted_at, name FROM hello WHERE id = ?",
            key
        )
        .fetch_optional(&self.pool)
        .await?;
        let mut hello = if let Some(row) = result {
            Hello::from(row)
        } else {
            return Ok(None);
        };
        let result: Option<HelloStatusRow> = sqlx::query_as!(
            HelloStatusRow,
            "SELECT hello_id, created_at, updated_at, deleted_at, message FROM hello_status WHERE hello_id = ?",
            key
        )
        .fetch_optional(&self.pool)
        .await?;
        if let Some(row) = result {
            hello.status = Some(HelloStatus::from(row));
        };

        Ok(Some(hello))
    }

    pub async fn all(&mut self) -> Result<Vec<Hello>, Error> {
        let result: Vec<HelloRow> = sqlx::query_as!(
            HelloRow,
            "SELECT id, created_at, updated_at, deleted_at, name FROM hello WHERE deleted_at IS NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut hellos: Vec<Hello> = result.into_iter().map(|r| Hello::from(r)).collect();

        // This is really inefficient, but just a demo so meh.
        for h in &mut hellos {
            let result: Option<HelloStatusRow> = sqlx::query_as!(
                HelloStatusRow,
                "SELECT hello_id, created_at, updated_at, deleted_at, message FROM hello_status WHERE hello_id = ?",
                h.id
            )
            .fetch_optional(&self.pool)
            .await?;
            if let Some(row) = result {
                h.status = Some(HelloStatus::from(row));
            };
        }

        Ok(hellos)
    }

    pub async fn all_deleted(&mut self, age: Duration) -> Result<Vec<Hello>, Error> {
        let deleted_before = Utc::now() - age;
        let result: Vec<HelloRow> = sqlx::query_as!(
            HelloRow,
            "SELECT id, created_at, updated_at, deleted_at, name FROM hello WHERE deleted_at IS NOT NULL and deleted_at < ?", deleted_before
        )
        .fetch_all(&self.pool)
        .await?;
        let hellos: Vec<Hello> = result.into_iter().map(|r| Hello::from(r)).collect();

        Ok(hellos)
    }

    #[allow(unused_must_use)] // This should be idempotent. If it fails we try again anyways.
    pub async fn remove(&mut self, key: &u64) -> Result<Option<()>, Error> {
        // We have cleanup access, so we should hard delete the spec and any associated status
        // rows.
        sqlx::query!("DELETE FROM hello WHERE id = ?", key)
            .execute(&self.pool)
            .await;
        sqlx::query!("DELETE FROM hello_status WHERE hello_id = ?", key)
            .execute(&self.pool)
            .await;

        Ok(Some(()))
    }
}
