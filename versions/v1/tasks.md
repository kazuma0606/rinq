# RINQ v1.0 タスク管理

チェックボックスの凡例: `[ ]` 未着手 / `[x]` 完了

---

## Milestone 1: Cargo.toml & ディレクトリ準備

### タスク
- [x] `Cargo.toml` の `[package] name` を `rusted-ca` → `rinq` に変更
- [x] `Cargo.toml` の `version` を `1.0.0` に更新
- [x] `Cargo.toml` の `[dependencies]` を最小化（`thiserror`, `num-traits`, `parking_lot` のみ）
- [x] `Cargo.toml` の `[dev-dependencies]` を整理（`proptest`, `criterion` のみ）
- [x] `Cargo.toml` の `[[bench]]` エントリを確認・更新
- [x] `src/core/` ディレクトリを作成
- [x] `src/metrics/` ディレクトリを作成
- [x] `src/lib.rs` を一時的に空（`// placeholder`）にして `cargo check` を通す
- [x] `build.rs` を削除（proto/gRPC ビルドスクリプト不要）
- [x] `src/main.rs` を削除（ライブラリクレートに移行）

### テスト確認
- [x] `cargo check` が通ること

### ✅ Milestone 1 完了

---

## Milestone 2: core モジュール実装

### タスク
- [x] `src/core/state.rs` を作成（既存 `src/domain/rinq/state.rs` をコピー、変更なし）
- [x] `src/core/error.rs` を作成（`RinqDomainError` → `RinqError` にリネーム）
- [x] `src/core/builder.rs` を作成（`src/domain/rinq/query_builder.rs` をベースに以下を修正）
  - [x] `use super::state::` → `use crate::core::state::`
  - [x] doc examples 内 `use rusted_ca::domain::rinq::` → `use rinq::`
- [x] `src/core/mod.rs` を作成（モジュール宣言と `pub use` re-export）
- [x] `src/lib.rs` に `pub mod core;` を追加し `core` の型を re-export

### テスト確認
- [x] `cargo check` が通ること
- [x] `cargo test --doc` が `core` の doc examples で通ること（24件 ok）

### ✅ Milestone 2 完了

---

## Milestone 3: metrics モジュール実装

### タスク
- [x] `src/metrics/collector.rs` を作成（既存 `src/shared/metrics/collector.rs` をコピー、変更なし）
- [x] `src/metrics/builder.rs` を作成（`src/domain/rinq/metrics_query_builder.rs` をベースに以下を修正）
  - [x] `use super::query_builder::QueryBuilder` → `use crate::core::builder::QueryBuilder`
  - [x] `use super::state::` → `use crate::core::state::`
  - [x] `use crate::shared::metrics::collector::MetricsCollector` → `use crate::metrics::collector::MetricsCollector`
  - [x] doc examples のクレートパス修正
- [x] `src/metrics/mod.rs` を作成（モジュール宣言と `pub use` re-export）
- [x] `src/lib.rs` に `pub mod metrics;` を追加し `MetricsQueryBuilder`, `MetricsCollector` を re-export

### テスト確認
- [x] `cargo check` が通ること
- [x] `cargo test --doc` が `metrics` の doc examples で通ること（25件 ok）

### ✅ Milestone 3 完了

---

## Milestone 4: lib.rs 公開 API 整備

### タスク
- [x] `src/lib.rs` を最終形に書き換え（全 re-export を整備）
  - [x] `pub use core::builder::{QueryBuilder, Queryable};`
  - [x] `pub use core::error::{RinqError, RinqResult};`
  - [x] `pub use core::state::{Filtered, Initial, Projected, Sorted};`
  - [x] `pub use metrics::builder::MetricsQueryBuilder;`
  - [x] `pub use metrics::collector::MetricsCollector;`
- [x] `use rinq::QueryBuilder` 形式でアクセスできることを確認（doc tests で検証済み）
- [x] `use rinq::core::builder::QueryBuilder` 形式でもアクセスできることを確認
- [x] `use rinq::metrics::MetricsCollector` 形式でもアクセスできることを確認

### テスト確認
- [x] `cargo check` が通ること
- [x] `cargo test --doc` が全 doc tests で通ること（25件 ok）

### ✅ Milestone 4 完了

---

## Milestone 5: テストスイート整備

### タスク
- [x] `tests/core_tests.rs` を作成（`src/domain/rinq/tests.rs` をコピーし以下を修正）
  - [x] `use crate::domain::rinq::QueryBuilder` → `use rinq::QueryBuilder`
  - [x] `use proptest::prelude::*` がファイル内に存在することを確認
- [x] `tests/rinq_property_tests.rs` の import を修正
  - [x] `use rusted_ca::domain::rinq::QueryBuilder` → `use rinq::QueryBuilder`
  - [x] `use rusted_ca::domain::rinq::query_builder::Queryable` → `use rinq::Queryable`
- [x] `tests/rinq_v0_2_tests.rs` の import を修正
  - [x] `use rusted_ca::domain::rinq::QueryBuilder` → `use rinq::QueryBuilder`
- [x] `tests/rinq_immutability_test.rs` の import を修正
  - [x] `use rusted_ca::domain::rinq::QueryBuilder` → `use rinq::QueryBuilder`
- [x] `tests/metrics_tests.rs` を新規作成（`tests/rinq_integration_tests.rs` をベースに以下を実施）
  - [x] import を `rinq::` ベースに修正
  - [x] `test_rinq_error_converts_to_application_error` を削除（`ApplicationError` 依存）
  - [x] `test_rinq_error_preserves_message` を削除（同上）

### テスト確認
- [x] `cargo test` が全テスト通ること（テスト数の合計を記録: 262件）
- [x] `cargo test --test core_tests` が通ること
- [x] `cargo test --test rinq_property_tests` が通ること
- [x] `cargo test --test rinq_v0_2_tests` が通ること
- [x] `cargo test --test rinq_immutability_test` が通ること
- [x] `cargo test --test metrics_tests` が通ること

### ✅ Milestone 5 完了

---

## Milestone 6: ベンチマーク整備

### タスク
- [x] `benches/rinq_benchmarks.rs` の import を修正
  - [x] `use rusted_ca::domain::rinq::QueryBuilder` → `use rinq::QueryBuilder`
- [x] `benches/rinq_v0_2_benchmarks.rs` の import を修正
  - [x] `use rusted_ca::domain::rinq::QueryBuilder` → `use rinq::QueryBuilder`
- [x] `Cargo.toml` の `[[bench]]` エントリがファイル名と一致していることを確認

### テスト確認
- [x] `cargo bench --no-run` が通ること（実際のベンチマーク実行は不要）

### ✅ Milestone 6 完了

---

## Milestone 7: 不要ファイル削除

### タスク（削除）
- [x] `src/application/` を削除
- [x] `src/infrastructure/` を削除
- [x] `src/presentation/` を削除
- [x] `src/domain/` を削除（rinq 関連は M2/M3 で移行済み）
- [x] `src/shared/` を削除（metrics は M3 で移行済み）
- [x] `src/state/` を削除
- [x] `src/main.rs` を削除
- [x] `proto/` を削除
- [x] `build.rs` を削除
- [x] `docker-compose.yml` を削除
- [x] `ARCHETECTURE.MD` を削除
- [x] `ARCHETECTURE2.MD` を削除
- [x] `ERRORTYPE.MD` を削除
- [x] `tests/auth_integration_test.rs` を削除
- [x] `tests/user_integration_test.rs` を削除
- [x] `tests/user_validation_test.rs` を削除
- [x] `tests/rinq_integration_tests.rs` を削除（`metrics_tests.rs` に置き換え済み）
- [x] `examples/test_rinq_filtering.rs` を削除

### テスト確認（回帰確認）
- [x] `cargo test` が引き続き全テスト通ること
- [x] `cargo check` が通ること

### ✅ Milestone 7 完了

---

## Milestone 8: ドキュメント整備

### タスク
- [x] `CHANGELOG.md` に v1.0 エントリを追加
  - [x] Breaking Changes（`RinqError` への改名、インポートパス変更）を明記
  - [x] 削除されたコードの記載
- [x] `CLAUDE.md` を新構造（`rinq` クレート）に合わせて更新
  - [x] コマンド欄（build, test, bench）を更新
  - [x] アーキテクチャ説明を新モジュール構成に更新
- [x] `examples/rinq_basic_usage.rs` は既に `use rinq::` ベース（変更不要）
- [x] `versions/v1/spec.md` の内容と実装が乖離していないか最終確認

### テスト確認
- [x] `cargo test` が通ること
- [x] `cargo test --doc` が通ること
- [x] `cargo doc --no-deps` がエラーなく生成されること

### ✅ Milestone 8 完了

---

## 全体完了チェック

- [x] `cargo test` 全件通過（テスト数: 262件）
- [x] `cargo bench --no-run` 通過
- [x] `cargo doc --no-deps` 通過
- [x] `cargo clippy -- -D warnings` の警告が許容範囲内（0件）
- [x] `versions/v1/spec.md` と実装の整合性確認
- [x] git commit 済み

### 🎉 RINQ v1.0 リリース完了
