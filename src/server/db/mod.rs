use sqlx::SqlitePool;

mod user;
mod worker;

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
