use super::Transaction;
use crate::common::domain::WorkerPublicKey;
use crate::server::domain::{UserId, Worker, WorkerId};
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

impl<const WRITE: bool> Transaction<WRITE> {
    pub async fn worker_get(&mut self, id: WorkerId) -> Result<Option<Worker>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM workers
                WHERE id = ?
            "#,
            id.0,
        )
        .fetch_optional(&mut *self.tx)
        .await
        .map(|row| row.map(Row::to_worker))
    }

    pub async fn worker_get_by_pubkey(
        &mut self, owner: UserId, key: WorkerPublicKey,
    ) -> Result<Option<Worker>, sqlx::Error> {
        let key = key.to_string();
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM workers
                WHERE owner = ? AND key = ?
            "#,
            owner.0,
            key,
        )
        .fetch_optional(&mut *self.tx)
        .await
        .map(|row| row.map(Row::to_worker))
    }

    pub fn worker_get_all(&mut self) -> impl Stream<Item = Result<Worker, sqlx::Error>> {
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM workers
            "#,
        )
        .fetch(&mut *self.tx)
        .map(|row| row.map(Row::to_worker))
    }

    pub fn worker_owned_by<'a, 'b, 'c>(
        &'a mut self, owner: &'b UserId,
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
        .fetch(&mut *self.tx)
        .map(|row| row.map(Row::to_worker))
    }
}

impl Transaction<true> {
    pub async fn worker_new(
        &mut self, owner: UserId, name: &str, key: WorkerPublicKey,
    ) -> Result<WorkerId, sqlx::Error> {
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
        .execute(&mut *self.tx)
        .await
        .map(|r| WorkerId(r.last_insert_rowid()))
    }

    pub async fn worker_set_enabled(&mut self, id: WorkerId, enabled: bool) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE workers
                SET enabled = ?
                WHERE id = ?
            "#,
            enabled,
            id.0,
        )
        .execute(&mut *self.tx)
        .await
        .map(|r| r.rows_affected() > 0)
    }
}
