# Implementation Plan

- [x] 1. プロジェクト構造とコア型の設定





  - RINQモジュールの作成（`src/domain/rinq/`）
  - 基本的な型定義（QueryBuilder、State型）
  - エラー型の定義（RinqDomainError）
  - _Requirements: 6.4, 11.1_

- [x] 1.1 プロパティテスト: 型状態パターンの検証


  - **Property 6.4: 型状態パターンによる有効なクエリ構築の強制**
  - **Validates: Requirements 6.4**

- [x] 2. QueryBuilder基本実装





  - `from()`メソッドの実装
  - `Initial`状態の実装
  - イテレータラッパーの実装
  - _Requirements: 1.1_

- [x] 2.1 単体テスト: from()の基本動作


  - Vec、スライスからのQueryBuilder作成
  - _Requirements: 1.1_

- [x] 2.2 プロパティテスト: from()の不変性


  - **Property 3: 不変性の保証**
  - **Validates: Requirements 1.5**

- [x] 3. フィルタリング機能の実装





  - `where_()`メソッドの実装
  - `Filtered`状態の実装
  - 述語の連鎖サポート
  - _Requirements: 1.2, 1.3_

- [x] 3.1 プロパティテスト: where_()の正確性


  - **Property 1: フィルタリングの正確性**
  - **Validates: Requirements 1.2**

- [x] 3.2 プロパティテスト: 複数フィルタの結合

  - **Property 2: 複数フィルタの結合**
  - **Validates: Requirements 1.3**

- [x] 4. 射影機能の実装












  - `select()`メソッドの実装
  - `Projected<U>`状態の実装
  - 型変換のサポート
  - _Requirements: 2.1, 2.3, 2.4_

- [x] 4.1 プロパティテスト: select()の正確性




  - **Property 4: 射影の正確性**
  - **Validates: Requirements 2.1**

- [x] 4.2 プロパティテスト: フィルタと射影の順序


  - **Property 5: フィルタと射影の順序**
  - **Validates: Requirements 2.2**

- [x] 4.3 単体テスト: 型変換のサポート


  - 異なる型への射影
  - _Requirements: 2.3, 2.4_

- [x] 5. ソート機能の実装



  - `order_by()`メソッドの実装
  - `order_by_descending()`メソッドの実装
  - `then_by()`メソッドの実装
  - `Sorted`状態の実装
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 5.1 プロパティテスト: 昇順ソートの正確性
  - **Property 7: 昇順ソートの正確性**
  - **Validates: Requirements 3.1**

- [x] 5.2 プロパティテスト: 降順ソートの正確性
  - **Property 8: 降順ソートの正確性**
  - **Validates: Requirements 3.2**

- [x] 5.3 プロパティテスト: 複数キーソートの正確性
  - **Property 9: 複数キーソートの正確性**
  - **Validates: Requirements 3.3, 3.4**

- [x] 5.4 プロパティテスト: 安定ソートの保証
  - **Property 10: 安定ソートの保証**
  - **Validates: Requirements 3.5**

- [x] 6. ページネーション機能の実装
  - `take()`メソッドの実装
  - `skip()`メソッドの実装
  - 遅延評価の実装
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 6.1 プロパティテスト: take()の正確性
  - **Property 11: take()の正確性**
  - **Validates: Requirements 4.1**

- [x] 6.2 プロパティテスト: skip()の正確性
  - **Property 12: skip()の正確性**
  - **Validates: Requirements 4.2**

- [x] 6.3 プロパティテスト: ページネーションの正確性
  - **Property 13: ページネーションの正確性**
  - **Validates: Requirements 4.3**

- [x] 6.4 単体テスト: エッジケース（nがサイズを超える）
  - _Requirements: 4.4_

- [x] 6.5 プロパティテスト: 遅延評価の保証
  - **Property 6: 遅延評価の保証**
  - **Validates: Requirements 2.5, 4.5**

- [x] 7. 集約機能の実装
  - `count()`メソッドの実装
  - `first()`メソッドの実装
  - `last()`メソッドの実装
  - `any()`メソッドの実装
  - `all()`メソッドの実装
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 7.1 プロパティテスト: count()の正確性
  - **Property 14: count()の正確性**
  - **Validates: Requirements 5.1**

- [x] 7.2 プロパティテスト: first()の正確性
  - **Property 15: first()の正確性**
  - **Validates: Requirements 5.2**

- [x] 7.3 プロパティテスト: last()の正確性
  - **Property 16: last()の正確性**
  - **Validates: Requirements 5.3**

- [x] 7.4 プロパティテスト: any()の正確性
  - **Property 17: any()の正確性**
  - **Validates: Requirements 5.4**

- [x] 7.5 プロパティテスト: all()の正確性
  - **Property 18: all()の正確性**
  - **Validates: Requirements 5.5**

- [x] 8. 終端操作の実装
  - `collect()`メソッドの実装
  - すべての状態での終端操作サポート
  - _Requirements: 1.4_

- [x] 8.1 単体テスト: collect()の基本動作
  - Vec、HashSet、その他のコレクションへの変換
  - _Requirements: 1.4_

- [ ] 9. デバッグ機能の実装
  - `inspect()`メソッドの実装
  - デバッグモードのサポート
  - _Requirements: 12.1, 12.2_

- [ ] 9.1 プロパティテスト: inspect()の非破壊性
  - **Property 19: inspect()の非破壊性**
  - **Validates: Requirements 12.2**

- [ ] 9.2 単体テスト: デバッグログの記録
  - _Requirements: 12.1_

- [ ] 10. Checkpoint - すべてのテストが通ることを確認
  - すべてのテストが通ることを確認し、問題があればユーザーに質問する

- [ ] 11. Queryableトレイトの実装
  - Queryableトレイトの定義
  - Vec<T>への実装
  - &[T]への実装
  - その他の標準コレクションへの実装
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ] 11.1 単体テスト: 借用データのサポート
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ] 12. エラーハンドリングの実装
  - RinqDomainErrorの実装
  - ApplicationErrorへの変換
  - Result型の使用
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [ ] 12.1 単体テスト: エラーハンドリング
  - 空コレクションでのfirst()
  - Result型の使用
  - _Requirements: 9.3, 9.4_

- [ ] 13. rusted-ca統合
  - DIコンテナへの統合
  - MetricsCollectorとの統合
  - MetricsQueryBuilderの実装
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [ ] 13.1 統合テスト: DIコンテナとの統合
  - _Requirements: 11.4_

- [ ] 13.2 統合テスト: メトリクス収集
  - _Requirements: 11.2_

- [ ] 13.3 単体テスト: エラー型の互換性
  - _Requirements: 11.3_

- [ ] 14. パフォーマンス最適化
  - インライン化の適用
  - イテレータ融合の確認
  - 不要な割り当ての削除
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 14.1 ベンチマーク: ゼロコスト抽象化の検証
  - 手書きループとの比較
  - _Requirements: 7.2_

- [ ] 14.2 ベンチマーク: メモリ使用量の測定
  - _Requirements: 7.5_

- [ ] 15. ドキュメントの作成
  - APIドキュメント（docコメント）
  - README.md
  - クイックスタートガイド
  - 使用例
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [ ] 16. Final Checkpoint - すべてのテストが通ることを確認
  - すべてのテストが通ることを確認し、問題があればユーザーに質問する
