//application/usecases/list_users_usecase.rs
// ユーザー一覧ユースケース
// 2025/7/8

use crate::application::dto::user_response_dto::UserResponseDto;
use crate::domain::repository::user_query_repository::UserQueryRepositoryInterface;
use crate::domain::value_object::pagination::{PaginatedResult, PaginationParams};
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait ListUsersUsecaseInterface: Send + Sync {
    async fn execute(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> ApplicationResult<PaginatedResult<UserResponseDto>>;
}

pub struct ListUsersUseCase<T>
where
    T: UserQueryRepositoryInterface + Send + Sync,
{
    query_repository: Arc<T>,
}

impl<T> ListUsersUseCase<T>
where
    T: UserQueryRepositoryInterface + Send + Sync,
{
    pub fn new(query_repository: Arc<T>) -> Self {
        Self { query_repository }
    }
}

#[async_trait]
impl<T> ListUsersUsecaseInterface for ListUsersUseCase<T>
where
    T: UserQueryRepositoryInterface + Send + Sync,
{
    async fn execute(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> ApplicationResult<PaginatedResult<UserResponseDto>> {
        // 1. ページネーションパラメータの構築
        let pagination = PaginationParams {
            page: page.unwrap_or(1),
            limit: limit.unwrap_or(20),
        };

        // 2. ユーザー一覧取得
        let paginated_users = self
            .query_repository
            .find_all(pagination)
            .await
            .map_err(|e| {
                ApplicationError::Infrastructure(
                    crate::shared::error::infrastructure_error::InfrastructureError::ResourceUnavailable {
                        resource: "users".to_string(),
                        message: format!("{}", e),
                    },
                )
            })?;

        // 3. レスポンスDTOに変換
        let items: Vec<UserResponseDto> = paginated_users
            .data
            .into_iter()
            .map(|user| UserResponseDto {
                id: user.id().0.clone(),
                email: user.email().0.clone(),
                name: user.name().0.clone(),
                phone: user.phone().map(|p| p.0.clone()),
                birth_date: user.birth_date().map(|b| b.0.clone()),
            })
            .collect();

        Ok(PaginatedResult {
            data: items,
            pagination: paginated_users.pagination,
        })
    }
}
