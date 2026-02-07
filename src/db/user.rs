use super::Repository;
use crate::domain::{Password, User, UserId};
use futures::{Stream, stream::StreamExt};

impl Repository {
    pub async fn user_get(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let res = sqlx::query!(
            r#"
                SELECT id, username, password, enabled, admin, autoapprove, approver
                FROM users
                WHERE username = ?
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;
        if let Some(res) = res {
            Ok(Some(User {
                id: UserId(res.id),
                username: res.username,
                password: res.password.map(Password::from_hash),
                enabled: res.enabled != 0,
                admin: res.admin != 0,
                autoapprove: res.autoapprove != 0,
                approver: res.approver != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn user_get_all(&self) -> impl Stream<Item = Result<User, sqlx::Error>> {
        sqlx::query!(
            r#"
                SELECT id, username, password, enabled, admin, autoapprove, approver
                FROM users
            "#
        )
        .fetch(&self.pool)
        .map(|res| {
            res.map(|res| User {
                id: UserId(res.id),
                username: res.username,
                password: res.password.map(Password::from_hash),
                enabled: res.enabled != 0,
                admin: res.admin != 0,
                autoapprove: res.autoapprove != 0,
                approver: res.approver != 0,
            })
        })
    }

    pub async fn user_new(&self, username: &str) -> Result<UserId, sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO users (username)
                VALUES (?)
            "#,
            username
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
            username
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
            username
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
            username
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
            username
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
            username
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }
}
