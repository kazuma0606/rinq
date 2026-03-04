//domain/repository/user_command_repository.rs
// Command Repository トレイト
// 2025/7/8

use crate::domain::entity::user::User;
use crate::domain::value_object::{email::Email, user_id::UserId};
use crate::shared::error::infrastructure_error::InfrastructureResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait UserCommandRepositoryInterface: Send + Sync {
    // 基本的なCRUD操作
    async fn save(&self, user: &User) -> InfrastructureResult<()>;
    async fn update(&self, user: &User) -> InfrastructureResult<()>;
    async fn delete(&self, user_id: &UserId) -> InfrastructureResult<()>;

    // トランザクション的操作
    async fn save_batch(&self, users: &[User]) -> InfrastructureResult<()>;
    async fn update_last_login(
        &self,
        user_id: &UserId,
        login_time: DateTime<Utc>,
    ) -> InfrastructureResult<()>;

    // 重複チェック用
    async fn exists_by_email(&self, email: &Email) -> InfrastructureResult<bool>;
}
