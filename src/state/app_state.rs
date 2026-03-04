//state/app_state.rs
// アプリケーション状態管理
// 2025/7/8

use crate::application::usecases::create_user_usecase::CreateUserUsecaseInterface;
use crate::application::usecases::delete_user_usecase::DeleteUserUsecaseInterface;
use crate::application::usecases::get_user_usecase::GetUserQueryUsecaseInterface;
use crate::application::usecases::list_users_usecase::ListUsersUsecaseInterface;
use crate::application::usecases::login_usecase::LoginUsecaseInterface;
use crate::application::usecases::update_user_usecase::UpdateUserUsecaseInterface;
use crate::shared::metrics::collector::MetricsCollector;
use std::sync::Arc;

/// アプリケーション全体の状態を保持する構造体
///
/// 責務:
/// 1. UseCaseインスタンスの保持（CQRS分離）
/// 2. メトリクス収集器の保持
/// 3. Axumルーターへの状態注入
///
/// 設計原則:
/// - トレイトオブジェクトを使用して具体実装から分離
/// - Command/Query UseCaseを明確に分離
/// - 不変性を保証（Arc<dyn Trait>）
#[derive(Clone)]
pub struct AppState {
    // ===== Command UseCases (Write Operations) =====
    pub create_user_usecase: Arc<dyn CreateUserUsecaseInterface>,
    pub update_user_usecase: Arc<dyn UpdateUserUsecaseInterface>,
    pub delete_user_usecase: Arc<dyn DeleteUserUsecaseInterface>,

    // ===== Query UseCases (Read Operations) =====
    pub get_user_usecase: Arc<dyn GetUserQueryUsecaseInterface>,
    pub list_users_usecase: Arc<dyn ListUsersUsecaseInterface>,

    // ===== Authentication UseCases =====
    pub login_usecase: Arc<dyn LoginUsecaseInterface>,

    // ===== Metrics & Monitoring =====
    pub metrics_collector: Arc<MetricsCollector>,
}

impl AppState {
    /// AppStateの新規作成
    ///
    /// # Arguments
    /// * `create_user_usecase` - ユーザー作成UseCase
    /// * `update_user_usecase` - ユーザー更新UseCase
    /// * `delete_user_usecase` - ユーザー削除UseCase
    /// * `get_user_usecase` - ユーザー取得UseCase
    /// * `list_users_usecase` - ユーザー一覧取得UseCase
    /// * `login_usecase` - ログインUseCase
    /// * `metrics_collector` - メトリクス収集器
    pub fn new(
        create_user_usecase: Arc<dyn CreateUserUsecaseInterface>,
        update_user_usecase: Arc<dyn UpdateUserUsecaseInterface>,
        delete_user_usecase: Arc<dyn DeleteUserUsecaseInterface>,
        get_user_usecase: Arc<dyn GetUserQueryUsecaseInterface>,
        list_users_usecase: Arc<dyn ListUsersUsecaseInterface>,
        login_usecase: Arc<dyn LoginUsecaseInterface>,
        metrics_collector: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            create_user_usecase,
            update_user_usecase,
            delete_user_usecase,
            get_user_usecase,
            list_users_usecase,
            login_usecase,
            metrics_collector,
        }
    }
}
