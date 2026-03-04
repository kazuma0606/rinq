//infrastructure/repository/in_memory_user_command_repository.rs
// SQLite Command Repository実装
// 2025/7/8

use crate::domain::entity::user::User;
use crate::domain::repository::user_command_repository::UserCommandRepositoryInterface;
use crate::domain::value_object::{email::Email, user_id::UserId};
use crate::infrastructure::database::sqlite_connection::SqliteConnection;
use crate::shared::error::infrastructure_error::{InfrastructureError, InfrastructureResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::params;

pub struct SqliteUserCommandRepository {
    db: SqliteConnection,
}

impl SqliteUserCommandRepository {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserCommandRepositoryInterface for SqliteUserCommandRepository {
    async fn save(&self, user: &User) -> InfrastructureResult<()> {
        println!(
            "SqliteUserCommandRepository: Saving user with ID: {}",
            user.id.0
        );
        let user = user.clone();
        let result: Result<(), rusqlite::Error> = self.db.execute_command(move |conn| {
            println!("SqliteUserCommandRepository: Executing INSERT query...");
            conn.execute(
                "INSERT INTO users (id, email, name, password, phone, birth_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    user.id.0,
                    user.email.0,
                    user.name.0,
                    user.password.0,
                    user.phone.as_ref().map(|p| p.0.clone()),
                    user.birth_date.as_ref().map(|b| b.0.clone()),
                ],
            )?;
            println!("SqliteUserCommandRepository: INSERT query executed successfully");
            Ok(())
        }).await;
        result.map_err(|e| {
            println!("SqliteUserCommandRepository: Error saving user: {}", e);
            InfrastructureError::DatabaseQuery {
                query: "INSERT INTO users".to_string(),
                message: e.to_string(),
            }
        })
    }

    async fn update(&self, user: &User) -> InfrastructureResult<()> {
        let user = user.clone();
        let result: Result<(), rusqlite::Error> = self.db.execute_command(move |conn| {
            conn.execute(
                "UPDATE users SET email = ?2, name = ?3, password = ?4, phone = ?5, birth_date = ?6 WHERE id = ?1",
                params![
                    user.id.0,
                    user.email.0,
                    user.name.0,
                    user.password.0,
                    user.phone.as_ref().map(|p| p.0.clone()),
                    user.birth_date.as_ref().map(|b| b.0.clone()),
                ],
            )?;
            Ok(())
        }).await;
        result.map_err(|e| InfrastructureError::DatabaseQuery {
            query: "UPDATE users".to_string(),
            message: e.to_string(),
        })
    }

    async fn delete(&self, user_id: &UserId) -> InfrastructureResult<()> {
        let user_id = user_id.clone();
        let result: Result<(), rusqlite::Error> = self
            .db
            .execute_command(move |conn| {
                conn.execute("DELETE FROM users WHERE id = ?", params![user_id.0])?;
                Ok(())
            })
            .await;
        result.map_err(|e| InfrastructureError::DatabaseQuery {
            query: "DELETE FROM users".to_string(),
            message: e.to_string(),
        })
    }

    async fn save_batch(&self, users: &[User]) -> InfrastructureResult<()> {
        let users = users.to_vec();
        let result: Result<(), rusqlite::Error> = self.db.execute_command(move |conn| {
            let tx = conn.transaction()?;
            for user in &users {
                tx.execute(
                    "INSERT INTO users (id, email, name, password, phone, birth_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        user.id.0.clone(),
                        user.email.0.clone(),
                        user.name.0.clone(),
                        user.password.0.clone(),
                        user.phone.as_ref().map(|p| p.0.clone()),
                        user.birth_date.as_ref().map(|b| b.0.clone()),
                    ],
                )?;
            }
            tx.commit()?;
            Ok(())
        }).await;
        result.map_err(|e| InfrastructureError::DatabaseTransaction {
            message: e.to_string(),
        })
    }

    async fn update_last_login(
        &self,
        user_id: &UserId,
        login_time: DateTime<Utc>,
    ) -> InfrastructureResult<()> {
        let user_id = user_id.clone();
        let login_time = login_time.clone();
        let result: Result<(), rusqlite::Error> = self
            .db
            .execute_command(move |conn| {
                conn.execute(
                    "UPDATE users SET last_login_at = ? WHERE id = ?",
                    params![login_time.to_rfc3339(), user_id.0],
                )?;
                Ok(())
            })
            .await;
        result.map_err(|e| InfrastructureError::DatabaseQuery {
            query: "UPDATE users last_login".to_string(),
            message: e.to_string(),
        })
    }

    async fn exists_by_email(&self, email: &Email) -> InfrastructureResult<bool> {
        let email = email.clone();
        let result: Result<bool, rusqlite::Error> = self
            .db
            .execute_query(move |conn| {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM users WHERE email = ?",
                    params![email.0],
                    |row| row.get(0),
                )?;
                Ok(count > 0)
            })
            .await;
        result.map_err(|e| InfrastructureError::DatabaseQuery {
            query: "SELECT COUNT FROM users".to_string(),
            message: e.to_string(),
        })
    }
}
