# RINQ v3.0 テスト戦略

**作成日**: 2026-03-25

---

## テスト全体方針

RINQ v2.0 までのテスト体系（単体テスト・統合テスト・プロパティテスト・doc test）を継承し、v3.0 では以下を追加します。

| 追加種別 | 目的 |
|---------|------|
| **E2E シナリオテスト** | 現実に近いデータと複合クエリで v2.0 実装の完全性を検証 |
| **並列テスト** | `ParallelQueryBuilder` の結果が逐次版と一致することを確認 |
| **ウィンドウテスト** | `moving_average` の `None` 位置・`rank_by` の同値処理を検証 |
| **失敗許容テスト** | `collect_partitioned` / `collect_results` の分岐ロジックを検証 |
| **統計テスト** | 既知の統計値と数値比較（許容誤差 1e-9） |

---

## v2.0 E2E シナリオテスト（実装済み機能の検証）

テストファイル: `tests/rinq_e2e_scenarios.rs`

### シナリオ設計の考え方

各シナリオは「現実に近いユースケース」を想定し、**複数の演算子を組み合わせた複合クエリ**をエンドツーエンドで検証します。個々のメソッドの単体テストではなく、メソッドが連鎖したときの振る舞いに焦点を当てます。

---

### Scenario 1: 売上データ分析パイプライン

**想定ユースケース**: ECサイトの注文データを集計・分析する

```
データ: Vec<Order> { product_id, category, amount, is_shipped }

クエリチェーン:
1. shipped のみフィルタ（where_）
2. カテゴリ別に集計（group_by + group_by_aggregate）
3. 売上トップ3カテゴリを抽出（order_by_descending + take）
4. カテゴリ名と合計を Vec<(String, f64)> で取得（select + collect）
```

検証ポイント:
- `group_by_aggregate` の集計結果が手動計算と一致すること
- `order_by_descending` + `take` でトップ N が正しく取れること
- `is_shipped = false` のデータが完全に除外されること

---

### Scenario 2: ログ解析パイプライン

**想定ユースケース**: アプリケーションログからエラー傾向を分析する

```
データ: Vec<LogEntry> { level: LogLevel, message: String, user_id: Option<u32> }

クエリチェーン:
1. ERROR / WARN レベルのみフィルタ（where_）
2. レベル別にカウント（group_by + group_by_aggregate）
3. 重複メッセージを除去（distinct_by）
4. ユーザーIDが存在するエントリのみ（flat_map + Option）
5. ユーザーIDの集合を取得（select + distinct + collect）
```

検証ポイント:
- `flat_map` で `Option<u32>` を `u32` に展開できること
- `distinct_by` がメッセージ文字列での重複除去に機能すること
- INFO レベルのログが結果に含まれないこと

---

### Scenario 3: 在庫管理 — 集合演算パイプライン

**想定ユースケース**: 2つの倉庫の在庫差分を管理する

```
データ: warehouse_a: Vec<ProductId>, warehouse_b: Vec<ProductId>

クエリチェーン:
1. 両倉庫に共通の商品（intersect）
2. 倉庫Aのみにある商品（except）
3. 両倉庫の全商品（union）
4. 倉庫A + 追加入荷予定（concat）→ 重複除去（distinct）
```

検証ポイント:
- `intersect` の結果が手動計算した積集合と一致すること
- `except` の結果に倉庫Bのみの商品が含まれないこと
- `union` の要素数が `intersect` + `except(A-B)` + `except(B-A)` の合計と一致すること（集合の分配法則）

---

### Scenario 4: ページング付き検索

**想定ユースケース**: REST API のページング処理

```
データ: Vec<User> { id, name, age, active }

クエリチェーン:
1. active ユーザーのみ（where_）
2. 名前順ソート（order_by）
3. ページ 2（20件ずつ）を取得（skip + take）
4. IDと名前のペア（select + collect）
5. 同一データで element_at による個別アクセス
```

検証ポイント:
- `skip(20).take(20)` が正確に 20 件目〜39 件目を返すこと
- `order_by` でソート後の `element_at(0)` が最小値の要素であること
- `skip` が要素数を超えた場合に空コレクションを返すこと

---

### Scenario 5: 複合集計 — 統計的サマリー

**想定ユースケース**: センサーデータの統計サマリーを生成する

```
データ: Vec<SensorReading> { sensor_id: u8, temperature: f64, timestamp: u64 }

クエリチェーン:
1. 異常値除外（where_ で -50.0..=100.0 の範囲外を除去）
2. センサーIDでグループ化（group_by）
3. 各グループで min / max / average / count を計算
4. 最も平均温度が高いセンサーを特定（max_by）
5. aggregate で全センサーの総観測数を集計
```

検証ポイント:
- `min`/`max` が `where_` フィルタ後のデータ範囲内に収まること
- `group_by` → 各グループへの `average` 計算が浮動小数点精度内で正しいこと
- `max_by` で返される要素が実際に最大平均を持つセンサーであること

---

### Scenario 6: zip による時系列比較

**想定ユースケース**: 前年同期比の計算

```
データ: current_year: Vec<f64>, previous_year: Vec<f64>（月次売上）

クエリチェーン:
1. zip で月ごとにペアリング（zip）
2. 前年比を計算（select → (current, previous, ratio)）
3. 前年比が 1.1 以上の月のみ抽出（where_）
4. 月インデックスを付与（enumerate → (index, ratio)）
5. 月番号と成長率の Vec を収集
```

検証ポイント:
- `zip` で長さが異なる場合に短い方で終了すること
- `enumerate` の index が 0 から始まること
- `select` で tuple を分解・再構成できること

---

### Scenario 7: エラーハンドリングの網羅

**想定ユースケース**: エラーパスが設計通りに機能することの確認

```
1. 空コレクションへの single() → Err(IteratorExhausted)
2. 複数要素への single() → Err(ExecutionError)
3. to_hashmap での重複キー → Err(ExecutionError)
4. 空コレクションへの first_or_default() → T::default()
5. single_or_default() の 0件/1件/複数件 の全ケース
```

検証ポイント:
- エラーバリアントが正確に一致すること（`RinqError::IteratorExhausted` vs `ExecutionError`）
- `first_or_default` / `last_or_default` がパニックしないこと
- `to_hashmap` の重複キー時に最初のエラーが適切に報告されること

---

### Scenario 8: 生成演算子 × 変換チェーン

**想定ユースケース**: テストデータ生成・数値シーケンス処理

```
1. range(0..100) でフィボナッチっぽい加工（aggregate）
2. repeat("ping", 5) → enumerate → select でプロトコルメッセージ生成
3. empty::<i32>() に concat で追記 → 通常クエリと同じ動作をすること
4. range(1..=10) .select(|x| x*x) .sum() == 385 の確認
```

検証ポイント:
- `QueryBuilder::empty` + `concat` が空から始まるクエリとして正しく機能すること
- `range` + `aggregate` での畳み込みが `fold` と同じ結果を出すこと

---

## v3.0 新機能のテスト設計方針

### 並列テスト（`tests/rinq_parallel_tests.rs`）

**基本方針**: 同一データ・同一クエリに対して逐次版と並列版の結果が一致すること。

プロパティテスト的アプローチ:
```
proptest! {
    fn par_sum_equals_sequential_sum(data: Vec<i32>) {
        let expected = QueryBuilder::from(data.clone()).sum();
        let actual   = ParallelQueryBuilder::from(data).par_sum();
        assert_eq!(expected, actual);
    }
}
```

### 統計テスト（`rinq-stats/tests/`）

**数値比較の許容誤差**: `f64` の精度制限を考慮し `(result - expected).abs() < 1e-9` を基準とする。

**既知のベンチマーク値**:
```
data = [2, 4, 4, 4, 5, 5, 7, 9]
  mean     = 5.0
  variance = 4.0
  std_dev  = 2.0

data = [1, 2, 3, 4, 5]
  median     = 3.0
  percentile(0.25) = 1.5 ～ 2.0（実装依存、一致させる）
```

### E2E テストのプロパティ化

複合クエリに対してプロパティを定義する例:
```
// group_by後の各グループ要素数の合計 = 元のコレクション要素数
forall data: Vec<T>:
    data.group_by(key).values().map(|v| v.len()).sum() == data.len()
```

---

## テストファイル一覧（v3.0 追加分）

| ファイル | 内容 |
|---------|------|
| `tests/rinq_e2e_scenarios.rs` | v2.0 機能の E2E 8シナリオ（本テスト文書） |
| `tests/rinq_parallel_tests.rs` | `ParallelQueryBuilder` の動作・逐次との一致確認 |
| `tests/rinq_window_tests.rs` | `running_sum`, `moving_average`, `rank_by`, `lag`, `lead` |
| `tests/rinq_try_tests.rs` | `try_select`, `collect_partitioned`, `collect_results` |
| `tests/rinq_serde_tests.rs` | `from_json`, `from_json_value`（`#[cfg(feature="serde")]`） |
| `rinq-stats/tests/statistics_tests.rs` | `StatisticsExt` の数値検証 |
| `rinq-stats/tests/pair_tests.rs` | `QueryPair` の共分散・相関係数・回帰 |
| `rinq-stats/tests/sampling_tests.rs` | サンプリング操作の統計的正確性 |
| `rinq-stats/tests/validation_tests.rs` | `ValidationExt` の違反収集ロジック |

---

## テスト実行コマンド

```bash
# v2.0 以前の全テスト（現時点で 350 件）
cargo test

# E2E シナリオのみ
cargo test --test rinq_e2e_scenarios

# feature flag 有効化テスト
cargo test --features parallel
cargo test --features serde
cargo test --all-features

# rinq-stats
cargo test -p rinq-stats

# doc テストのみ
cargo test --doc

# プロパティテストのみ（反復回数を増やして実行）
PROPTEST_CASES=10000 cargo test --test rinq_property_tests

# 全体
cargo test --all-features && cargo test -p rinq-stats
```

---

## リリース基準

| チェック項目 | コマンド | 期待結果 |
|------------|---------|---------|
| 全テスト通過 | `cargo test --all-features` | 0 failures |
| rinq-stats テスト通過 | `cargo test -p rinq-stats` | 0 failures |
| doc テスト通過 | `cargo test --doc --all-features` | 0 failures |
| Lint ゼロ | `cargo clippy --all-features -- -D warnings` | 0 warnings |
| ドキュメントビルド | `cargo doc --no-deps --all-features` | 0 errors |
| ベンチマーク | `cargo bench --no-run` | Compiles OK |
| 公開ドライラン | `cargo publish --dry-run` | OK |
