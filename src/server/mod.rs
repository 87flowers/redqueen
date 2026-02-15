pub mod db;
pub mod domain;
pub mod service;

pub async fn connect_to_repository() -> Result<db::Repository, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};

    let opts = SqliteConnectOptions::new()
        .filename("rqdatabase.db")
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    Ok(db::Repository::new(pool))
}
