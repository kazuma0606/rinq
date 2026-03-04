//application/usecases/login_usecase.rs
// ログインユースケース
// 2025/7/8

use crate::application::dto::user_request_dto::LoginRequestDto;
use crate::application::dto::user_response_dto::LoginResponseDto;
use crate::domain::repository::user_query_repository::UserQueryRepositoryInterface;
use crate::domain::value_object::email::Email;
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use crate::shared::utils::password_hasher::{PasswordHasher, SimplePasswordHasher};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait LoginUsecaseInterface: Send + Sync {
    async fn execute(&self, request_dto: LoginRequestDto) -> ApplicationResult<LoginResponseDto>;
}

pub struct LoginUseCase<T, P>
where
    T: UserQueryRepositoryInterface + Send + Sync,
    P: PasswordHasher + Send + Sync,
{
    query_repository: Arc<T>,
    password_hasher: Arc<P>,
}

impl<T, P> LoginUseCase<T, P>
where
    T: UserQueryRepositoryInterface + Send + Sync,
    P: PasswordHasher + Send + Sync,
{
    pub fn new(query_repository: Arc<T>, password_hasher: Arc<P>) -> Self {
        Self {
            query_repository,
            password_hasher,
        }
    }
}

#[async_trait]
impl<T, P> LoginUsecaseInterface for LoginUseCase<T, P>
where
    T: UserQueryRepositoryInterface + Send + Sync,
    P: PasswordHasher + Send + Sync,
{
    async fn execute(&self, request_dto: LoginRequestDto) -> ApplicationResult<LoginResponseDto> {
        // 1. メールアドレスのバリデーション
        let email = Email::new(request_dto.email.clone()).map_err(|e| {
            ApplicationError::ValidationFailed {
                field: "email".to_string(),
                message: e.to_string(),
            }
        })?;

        // 2. ユーザー取得
        let user = self
            .query_repository
            .find_by_email(&email)
            .await
            .map_err(|e| {
                ApplicationError::Infrastructure(
                    crate::shared::error::infrastructure_error::InfrastructureError::ResourceUnavailable {
                        resource: "user".to_string(),
                        message: format!("{}", e),
                    },
                )
            })?
            .ok_or(ApplicationError::AuthorizationFailed {
                message: "Invalid email or password".to_string(),
            })?;

        // 3. パスワード検証
        let is_valid = self
            .password_hasher
            .verify(&request_dto.password, &user.password().0)
            .map_err(|e| ApplicationError::AuthorizationFailed {
                message: format!("Password verification failed: {}", e),
            })?;

        if !is_valid {
            return Err(ApplicationError::AuthorizationFailed {
                message: "Invalid email or password".to_string(),
            });
        }

        // 4. JWTトークン生成（簡易実装）
        // TODO: 実際のJWT生成ロジックを実装
        let token = format!("jwt_token_for_{}", user.id().0);

        // 5. レスポンス生成
        Ok(LoginResponseDto {
            token,
            user_id: user.id().0.clone(),
            email: user.email().0.clone(),
            name: user.name().0.clone(),
        })
    }
}
