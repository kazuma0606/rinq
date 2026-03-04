//application/usecases/create_user_usecase.rs
// ユーザー作成ユースケース
// 2025/7/8

use crate::application::dto::user_request_dto::CreateUserRequestDto;
use crate::application::dto::user_response_dto::UserResponseDto;
use crate::domain::entity::user::User;
use crate::domain::repository::user_command_repository::UserCommandRepositoryInterface;
use crate::domain::service::id_generator::IdGeneratorInterface;
use crate::domain::value_object::{
    birth_date::BirthDate, email::Email, password::Password, phone::Phone, user_name::UserName,
};
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait CreateUserUsecaseInterface: Send + Sync {
    async fn execute(
        &self,
        request_dto: CreateUserRequestDto,
    ) -> ApplicationResult<UserResponseDto>;
}

pub struct CreateUserUseCase {
    command_repository: Arc<dyn UserCommandRepositoryInterface + Send + Sync>,
    id_generator: Arc<dyn IdGeneratorInterface + Send + Sync>,
}

impl CreateUserUseCase {
    pub fn new(
        command_repository: Arc<dyn UserCommandRepositoryInterface + Send + Sync>,
        id_generator: Arc<dyn IdGeneratorInterface + Send + Sync>,
    ) -> Self {
        Self {
            command_repository,
            id_generator,
        }
    }
}

#[async_trait]
impl CreateUserUsecaseInterface for CreateUserUseCase {
    async fn execute(
        &self,
        request_dto: CreateUserRequestDto,
    ) -> ApplicationResult<UserResponseDto> {
        // 1. バリデーション＆ドメイン変換
        let id = self.id_generator.generate_user_id();
        let email = Email::new(request_dto.email.clone()).map_err(|e| {
            ApplicationError::ValidationFailed {
                field: "email".to_string(),
                message: e.to_string(),
            }
        })?;
        let name = UserName::new(request_dto.name.clone()).map_err(|e| {
            ApplicationError::ValidationFailed {
                field: "name".to_string(),
                message: e.to_string(),
            }
        })?;
        let password = Password::new(request_dto.password.clone()).map_err(|e| {
            ApplicationError::ValidationFailed {
                field: "password".to_string(),
                message: e.to_string(),
            }
        })?;
        let phone = request_dto
            .phone
            .as_ref()
            .map(|p| Phone::new(p.clone()))
            .transpose()
            .map_err(|e| ApplicationError::ValidationFailed {
                field: "phone".to_string(),
                message: e.to_string(),
            })?;
        let birth_date = request_dto
            .birth_date
            .as_ref()
            .map(|b| BirthDate::new(b.clone()))
            .transpose()
            .map_err(|e| ApplicationError::ValidationFailed {
                field: "birth_date".to_string(),
                message: e.to_string(),
            })?;

        let user = User::new(id, email, name, password, phone, birth_date).map_err(|e| {
            ApplicationError::InvalidInput {
                input: "user".to_string(),
                reason: format!("{}", e),
            }
        })?;

        // 2. 保存
        self.command_repository
            .save(&user)
            .await
            .map_err(ApplicationError::Infrastructure)?;

        // 3. レスポンスDTO生成
        Ok(UserResponseDto {
            id: user.id.0,
            email: user.email.0,
            name: user.name.0,
            phone: user.phone.map(|p| p.0),
            birth_date: user.birth_date.map(|b| b.0),
        })
    }
}
