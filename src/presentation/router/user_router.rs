//presentation/router/user_router.rs
// ユーザー関連ルーティング
// 2025/7/8

use crate::state::app_state::AppState;
use crate::shared::middleware::auth_middleware::{AdminUser, AuthenticatedUser};
use axum::middleware::from_fn;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// ユーザー関連のルーティング設定
///
/// 責務:
/// 1. Axumルーターの設定
/// 2. エンドポイントとControllerメソッドのマッピング
/// 3. クロージャでのController呼び出し
pub fn create_user_routes(app_state: Arc<AppState>) -> Router
{
    Router::new()
        .route(
            "/users",
            post({
                let app_state = app_state.clone();
                move |_auth: AuthenticatedUser, _request: axum::Json<serde_json::Value>| {
                    let _app_state = app_state.clone();
                    async move { 
                        // TODO: Implement using app_state
                        Ok::<_, axum::http::StatusCode>(axum::Json(serde_json::json!({"message": "Not implemented"})))
                    }
                }
            }),
        )
        .route(
            "/users/:id",
            get({
                let app_state = app_state.clone();
                move |_path: axum::extract::Path<String>| {
                    let _app_state = app_state.clone();
                    async move { 
                        // TODO: Implement using app_state
                        Ok::<_, axum::http::StatusCode>(axum::Json(serde_json::json!({"message": "Not implemented"})))
                    }
                }
            }),
        )
        .route(
            "/users/:id",
            put({
                let app_state = app_state.clone();
                move |_auth: AuthenticatedUser, _path: axum::extract::Path<String>, _body: axum::Json<serde_json::Value>| {
                    let _app_state = app_state.clone();
                    async move { 
                        // TODO: Implement using app_state
                        Ok::<_, axum::http::StatusCode>(axum::Json(serde_json::json!({"message": "Not implemented"})))
                    }
                }
            }),
        )
        .route(
            "/users/:id",
            delete({
                let app_state = app_state.clone();
                move |_auth: AuthenticatedUser, _path: axum::extract::Path<String>| {
                    let _app_state = app_state.clone();
                    async move { 
                        // TODO: Implement using app_state
                        Ok::<_, axum::http::StatusCode>(axum::Json(serde_json::json!({"message": "Not implemented"})))
                    }
                }
            }),
        )
}
