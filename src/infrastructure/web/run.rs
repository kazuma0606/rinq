//infrastructure/web/run.rs
// Webサーバー実行関数
// 2025/7/8

use axum::{
    Router,
    routing::{get, post},
    serve,
};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::infrastructure::config::app_config::AppConfig;
use crate::infrastructure::di::container::DIContainer;
// use crate::infrastructure::grpc::server::create_grpc_router;
use crate::infrastructure::utils::graceful_shutdown::shutdown_signal;
use crate::presentation::router::app_router::create_app_router;

/// Webサーバーを起動する
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. DIコンテナの初期化とAppState構築
    let di_container = DIContainer::new();
    let app_state = di_container.build_app_state()?;
    println!("✅ AppStateの構築が完了しました");

    // 2. アプリケーション設定の読み込み
    let app_config = AppConfig::from_env();
    let discord_config = Arc::new(app_config.discord);

    // 3. ルーティング設定
    let http_router = create_app_router(Arc::new(app_state), discord_config);
    // let grpc_router = create_grpc_router();  // Temporarily disabled - requires protoc

    // HTTPとgRPCルーターを統合
    // let app = http_router.merge(grpc_router);
    let app = http_router;

    // 4. サーバー起動
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🚀 Server starting on {}", addr);
    println!("📋 利用可能なエンドポイント:");
    println!("  - POST /api/auth/login - ログイン(ユーザー認証)");
    println!("  - GET  /health - ヘルスチェック");
    println!("  - GET  /api/health - APIヘルスチェック");
    println!("  - POST /api/users - ユーザー作成");
    println!("  - GET  /api/users/:id - ユーザー取得");
    println!("  - PUT  /api/users/:id - ユーザー更新");
    println!("  - DELETE /api/users/:id - ユーザー削除");
    println!("  - GET  /api/fortune - ランダム癒し系おみくじ");
    // println!("  - POST /grpc/hello - gRPC Hello Service (Protocol Buffers)");  // Temporarily disabled
    println!("  - Discord通知: エラー発生時に自動通知");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// APIヘルスチェックエンドポイント
async fn api_health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "message": "API is running",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
