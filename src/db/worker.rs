use super::Repository;
use crate::domain::{UserId, Worker, WorkerId, WorkerPublicKey};
use futures::{Stream, StreamExt};

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    owner: i64,
    name: String,
    enabled: i64,
    key: String,
}

impl Row {
    fn to_worker(self) -> Worker {
        Worker {
            id: WorkerId(self.id),
            owner: UserId(self.owner),
            name: self.name,
            enabled: self.enabled != 0,
            key: WorkerPublicKey::from_str(&self.key).ok(),
        }
    }
}

impl Repository {
    pub async fn worker_get(&self, id: WorkerId) -> Result<Option<Worker>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM workers
                WHERE id = ?
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Row::to_worker))
    }

    pub fn worker_get_all(&self) -> impl Stream<Item = Result<Worker, sqlx::Error>> {
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM workers
            "#,
        )
        .fetch(&self.pool)
        .map(|row| row.map(Row::to_worker))
    }

    pub fn worker_owned_by<'a, 'b, 'c>(
        &'a self, owner: &'b UserId,
    ) -> impl Stream<Item = Result<Worker, sqlx::Error>> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM workers
                WHERE owner = ?
            "#,
            owner.0,
        )
        .fetch(&self.pool)
        .map(|row| row.map(Row::to_worker))
    }

    pub async fn worker_new(&self, owner: UserId, name: &str, key: WorkerPublicKey) -> Result<WorkerId, sqlx::Error> {
        let key = key.to_string();
        sqlx::query!(
            r#"
                INSERT INTO workers (owner, name, key)
                VALUES (?, ?, ?)
            "#,
            owner.0,
            name,
            key,
        )
        .execute(&self.pool)
        .await
        .map(|r| WorkerId(r.last_insert_rowid()))
    }

    pub async fn worker_set_enabled(&self, id: WorkerId, enabled: bool) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE workers
                SET enabled = ?
                WHERE id = ?
            "#,
            enabled,
            id.0,
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }
}
