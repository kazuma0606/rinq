# RINQ v1.0 実装計画

## 目標

`rusted-ca` クレート内に埋め込まれていた RINQ を独立したライブラリクレート `rinq` として整備する。
機能の追加・変更は行わず、**構造の再編と不要コードの削除のみ**を実施する。

---

## 設計方針

### 変更しないもの
- `QueryBuilder` のすべての公開メソッド（シグネチャ含む）
- `MetricsQueryBuilder` のすべての公開メソッド
- テストの検証内容（import パスのみ修正）
- ベンチマークのロジック（import パスのみ修正）

### 変更するもの
- クレート名: `rusted-ca` → `rinq`
- エラー型名: `RinqDomainError` → `RinqError`（"Domain" プレフィックスを削除）
- モジュールパス: `domain::rinq` → `core`、`shared::metrics` → `metrics`
- 削除: Web アプリ固有コード一切（presentation / application / infrastructure / domain entities / shared middleware 等）

---

## フェーズ構成

```
M1: Cargo.toml & ディレクトリ準備
  ↓
M2: core モジュール実装（state / error / builder）
  ↓
M3: metrics モジュール実装（collector / builder）
  ↓
M4: lib.rs 公開 API 整備
  ↓
M5: テストスイート整備
  ↓
M6: ベンチマーク整備
  ↓
M7: 不要ファイル削除
  ↓
M8: ドキュメント整備（CHANGELOG / CLAUDE.md / examples）
```

各マイルストーンは **単体・結合テスト通過を確認してから完了**とする。

---

## 各フェーズ詳細

### M1: Cargo.toml & ディレクトリ準備

**目的**: ビルド設定を `rinq` クレートとして更新し、新ディレクトリ骨格を作成する。

**作業**:
- `[package] name` を `rusted-ca` → `rinq` に変更
- version を `1.0.0` に更新
- 依存を最小化（`thiserror`, `num-traits`, `parking_lot` のみ）
- dev-dependencies: `proptest`, `criterion`
- `[[bench]]` エントリを更新
- `src/core/`, `src/metrics/`, `versions/v1/` ディレクトリを作成

**確認**: `cargo check` が通ること（既存 lib.rs は一時的に空にする）

---

### M2: core モジュール実装

**目的**: クエリエンジンの純粋なコアを `src/core/` に配置する。

**作業**:
- `src/core/state.rs` — 既存 `state.rs` をそのままコピー
- `src/core/error.rs` — `RinqDomainError` → `RinqError` にリネーム
- `src/core/builder.rs` — `query_builder.rs` をコピーし以下を修正:
  - `use super::state::` → `use crate::core::state::`
  - doc examples 内 `use rusted_ca::domain::rinq::` → `use rinq::`
- `src/core/mod.rs` — モジュール宣言と re-export

**確認**: `cargo check` が通ること

---

### M3: metrics モジュール実装

**目的**: `MetricsCollector` と `MetricsQueryBuilder` を `src/metrics/` に配置する。

**作業**:
- `src/metrics/collector.rs` — 既存 `shared/metrics/collector.rs` をそのままコピー
- `src/metrics/builder.rs` — `metrics_query_builder.rs` をコピーし以下を修正:
  - `use super::query_builder::QueryBuilder` → `use crate::core::builder::QueryBuilder`
  - `use super::state::` → `use crate::core::state::`
  - `use crate::shared::metrics::collector::MetricsCollector` → `use crate::metrics::collector::MetricsCollector`
  - doc examples のパス修正
- `src/metrics/mod.rs` — モジュール宣言と re-export

**確認**: `cargo check` が通ること

---

### M4: lib.rs 公開 API 整備

**目的**: クレートのエントリポイントを新構造に対応させ、利用者向けの re-export を整備する。

**作業**:
- `src/lib.rs` を全面書き換え:
  - `pub mod core;` / `pub mod metrics;`
  - トップレベル re-export（`QueryBuilder`, `Queryable`, `RinqError`, 状態型, `MetricsQueryBuilder`, `MetricsCollector`）

**確認**: `cargo check` が通ること / doc tests が通ること

---

### M5: テストスイート整備

**目的**: すべてのテストファイルを新インポートパスに対応させる。

**作業**:

| 元ファイル | 新ファイル | 変更内容 |
|-----------|-----------|---------|
| `src/domain/rinq/tests.rs` | `tests/core_tests.rs` | `use crate::` → `use rinq::` |
| `tests/rinq_property_tests.rs` | 同名（上書き） | import パス修正 |
| `tests/rinq_v0_2_tests.rs` | 同名（上書き） | import パス修正 |
| `tests/rinq_immutability_test.rs` | 同名（上書き） | import パス修正 |
| `tests/rinq_integration_tests.rs` | `tests/metrics_tests.rs` | import 修正 + `ApplicationError` 依存テスト削除 |

**確認**: `cargo test` が全テスト通ること

---

### M6: ベンチマーク整備

**目的**: ベンチマークファイルのインポートを新パスに修正する。

**作業**:
- `benches/rinq_benchmarks.rs`: `use rusted_ca::domain::rinq::` → `use rinq::`
- `benches/rinq_v0_2_benchmarks.rs`: 同上
- Cargo.toml の `[[bench]]` エントリ名を確認・整合

**確認**: `cargo bench --no-run` が通ること

---

### M7: 不要ファイル削除

**目的**: Web アプリ固有のコードをすべて削除し、ディレクトリを整理する。

**削除対象**:
```
src/application/
src/infrastructure/
src/presentation/
src/domain/              ← rinq 関連は M2/M3 で移行済み
src/shared/              ← metrics は M3 で移行済み
src/state/
src/main.rs
proto/
build.rs
docker-compose.yml
ARCHETECTURE.MD
ARCHETECTURE2.MD
ERRORTYPE.MD
tests/auth_integration_test.rs
tests/user_integration_test.rs
tests/user_validation_test.rs
tests/rinq_integration_tests.rs   ← metrics_tests.rs に置き換え済み
examples/test_rinq_filtering.rs
```

**確認**: `cargo test` が引き続き全テスト通ること（削除後の回帰確認）

---

### M8: ドキュメント整備

**目的**: ドキュメントと付随ファイルを v1.0 に合わせて更新する。

**作業**:
- `CHANGELOG.md` に v1.0 エントリを追加
- `CLAUDE.md` を新構造に合わせて更新
- `examples/basic_usage.rs` を `use rinq::` ベースに更新
- `src/domain/rinq/README.md` を `docs/` 等に移動（任意）

**確認**: `cargo test --doc` が通ること / `cargo doc` がエラーなく生成されること

---

## リスク・注意事項

### `RinqError` への名称変更
- `RinqDomainError` は `ApplicationError` への `From` 変換を持っていた（rusted-ca 固有）
- この変換は v1.0 で**削除**する（`ApplicationError` 自体が存在しない）
- テストの `test_rinq_error_converts_to_application_error` も削除対象

### doc tests
- `builder.rs` 内の doc examples は `use rusted_ca::domain::rinq::` を使用しているため、M2 で修正必須
- 修正漏れは `cargo test --doc` で検出できる

### benches の [[bench]] 名とファイル名の対応
- Cargo.toml の `name` と `benches/` 以下のファイル名（拡張子なし）が一致している必要がある
- ファイルをリネームする場合は Cargo.toml も合わせて更新すること
