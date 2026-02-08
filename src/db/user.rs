use super::Repository;
use crate::domain::{Password, User, UserId};
use futures::{Stream, stream::StreamExt};

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    username: String,
    password: Option<String>,
    enabled: i64,
    admin: i64,
    autoapprove: i64,
    approver: i64,
}

impl Row {
    fn to_user(self) -> User {
        User {
            id: UserId(self.id),
            username: self.username,
            password: self.password.map(Password::from_hash),
            enabled: self.enabled != 0,
            admin: self.admin != 0,
            autoapprove: self.autoapprove != 0,
            approver: self.approver != 0,
        }
    }
}

impl Repository {
    pub async fn user_get(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM users
                WHERE username = ?
            "#,
            username,
        )
        .fetch_optional(&self.pool)
        .await
        .map(|user| user.map(Row::to_user))
    }

    pub fn user_get_all(&self) -> impl Stream<Item = Result<User, sqlx::Error>> {
        sqlx::query_as!(
            Row,
            r#"
                SELECT *
                FROM users
            "#
        )
        .fetch(&self.pool)
        .map(|user| user.map(Row::to_user))
    }

    pub async fn user_new(&self, username: &str) -> Result<UserId, sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO users (username)
                VALUES (?)
            "#,
            username,
        )
        .execute(&self.pool)
        .await
        .map(|r| UserId(r.last_insert_rowid()))
    }

    pub async fn user_set_password(&self, username: &str, password: Password) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE users
                SET password = ?
                WHERE username = ?
            "#,
            password.0,
            username,
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }

    pub async fn user_set_enabled(&self, username: &str, enabled: bool) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE users
                SET enabled = ?
                WHERE username = ?
            "#,
            enabled,
            username,
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }

    pub async fn user_set_auto_approve(&self, username: &str, enabled: bool) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE users
                SET autoapprove = ?
                WHERE username = ?
            "#,
            enabled,
            username,
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }

    pub async fn user_set_approver(&self, username: &str, enabled: bool) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE users
                SET approver = ?
                WHERE username = ?
            "#,
            enabled,
            username,
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }

    pub async fn user_set_admin(&self, username: &str, enabled: bool) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE users
                SET admin = ?
                WHERE username = ?
            "#,
            enabled,
            username,
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }
}
