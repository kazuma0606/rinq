//infrastructure/di/container.rs
// DIコンテナ - CQRS対応（改善版）
// 2025/7/8

use crate::application::usecases::create_user_usecase::{
    CreateUserUseCase, CreateUserUsecaseInterface,
};
use crate::application::usecases::delete_user_usecase::{
    DeleteUserUseCase, DeleteUserUsecaseInterface,
};
use crate::application::usecases::get_user_usecase::{
    GetUserQueryUsecaseInterface, GetUserUseCase,
};
use crate::application::usecases::list_users_usecase::{
    ListUsersUseCase, ListUsersUsecaseInterface,
};
use crate::application::usecases::login_usecase::{LoginUseCase, LoginUsecaseInterface};
use crate::application::usecases::update_user_usecase::{
    UpdateUserUseCase, UpdateUserUsecaseInterface,
};
use crate::domain::service::id_generator::{IdGeneratorInterface, UuidGenerator};
use crate::infrastructure::database::sqlite_connection::SqliteConnection;
use crate::infrastructure::repository::in_memory_user_command_repository::SqliteUserCommandRepository;
use crate::infrastructure::repository::in_memory_user_query_repository::SqliteUserQueryRepository;
use crate::shared::metrics::collector::MetricsCollector;
use crate::shared::utils::password_hasher::SimplePasswordHasher;
use crate::state::app_state::AppState;
use std::sync::Arc;

/// DIコンテナ
///
/// 責務:
/// 1. 依存関係の組み立て
/// 2. Repository実装の注入
/// 3. UseCaseの組み立て
/// 4. AppStateの構築
/// 5. CQRSパターンの実装
///
/// 設計原則:
/// - トレイトオブジェクトを使用して具体実装から分離
/// - 型の複雑さを隠蔽
/// - テスト可能な設計
pub struct DIContainer;

impl DIContainer {
    pub fn new() -> Self {
        Self
    }

    /// AppStateを構築して返す
    ///
    /// これがDIコンテナのメインエントリーポイント
    pub fn build_app_state(
        &self,
    ) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Infrastructure Layer - データベース接続
        let db_connection = SqliteConnection::new_in_memory()?;

        // 2. Infrastructure Layer - Repositories (CQRS分離)
        let command_repository = Arc::new(SqliteUserCommandRepository::new(db_connection.clone()));
        let query_repository = Arc::new(SqliteUserQueryRepository::new(db_connection));

        // トレイトオブジェクトに変換
        let command_repo_trait: Arc<
            dyn crate::domain::repository::user_command_repository::UserCommandRepositoryInterface
                + Send
                + Sync,
        > = command_repository;
        let query_repo_trait: Arc<
            dyn crate::domain::repository::user_query_repository::UserQueryRepositoryInterface
                + Send
                + Sync,
        > = query_repository.clone();

        // 3. Domain Layer - ID生成器
        let id_generator = Arc::new(UuidGenerator);

        // 4. Shared Layer - ユーティリティ
        let password_hasher = Arc::new(SimplePasswordHasher::new());
        let metrics_collector = Arc::new(MetricsCollector::new());

        // 5. Application Layer - UseCases (Command側)
        let create_user_usecase: Arc<dyn CreateUserUsecaseInterface> = Arc::new(
            CreateUserUseCase::new(command_repo_trait.clone(), id_generator.clone()),
        );

        let update_user_usecase: Arc<dyn UpdateUserUsecaseInterface> = Arc::new(
            UpdateUserUseCase::new(command_repo_trait.clone(), query_repo_trait.clone()),
        );

        let delete_user_usecase: Arc<dyn DeleteUserUsecaseInterface> = Arc::new(
            DeleteUserUseCase::new(command_repo_trait, query_repo_trait.clone()),
        );

        // 6. Application Layer - UseCases (Query側)
        let get_user_usecase: Arc<dyn GetUserQueryUsecaseInterface> =
            Arc::new(GetUserUseCase::new(query_repository.clone()));

        let list_users_usecase: Arc<dyn ListUsersUsecaseInterface> =
            Arc::new(ListUsersUseCase::new(query_repository.clone()));

        // 7. Application Layer - Authentication UseCase
        let login_usecase: Arc<dyn LoginUsecaseInterface> =
            Arc::new(LoginUseCase::<_, SimplePasswordHasher>::new(query_repository, password_hasher));

        // 8. AppState構築
        let app_state = AppState::new(
            create_user_usecase,
            update_user_usecase,
            delete_user_usecase,
            get_user_usecase,
            list_users_usecase,
            login_usecase,
            metrics_collector,
        );

        Ok(app_state)
    }

    /// テスト用: 依存関係の検証
    pub fn verify_dependencies(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔍 DIコンテナの依存関係を検証中...");

        // AppStateの構築を試みる
        let app_state = self.build_app_state()?;

        println!("✅ AppState構築成功");
        println!("✅ Command UseCases: 3個");
        println!("✅ Query UseCases: 2個");
        println!("✅ Authentication UseCases: 1個");
        println!("✅ Metrics Collector: 1個");
        println!("✅ 全ての依存関係が正常に解決されました");

        Ok(())
    }
}

impl Default for DIContainer {
    fn default() -> Self {
        Self::new()
    }
}
