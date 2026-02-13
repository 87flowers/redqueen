use sqlx::{Sqlite, SqlitePool};

mod user;
mod worker;

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn begin_read(&self) -> Result<Transaction<false>, sqlx::Error> {
        self.pool.begin().await.map(|tx| Transaction { tx })
    }

    pub async fn begin_write(&self) -> Result<Transaction<true>, sqlx::Error> {
        self.pool.begin_with("BEGIN IMMEDAITE").await.map(|tx| Transaction { tx })
    }
}

pub struct Transaction<const WRITE: bool> {
    tx: sqlx::Transaction<'static, Sqlite>,
}
