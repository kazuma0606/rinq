# Requirements Document

## Introduction

RINQ (Rust Integrated Query) v0.1は、Rust向けの型安全・ゼロコスト・宣言的なクエリエンジンの初期バージョンです。C#のLINQから着想を得つつ、Rustの型システムとパフォーマンス特性を最大限に活かします。

v0.1では、In-Memoryコレクション（Vec、スライス）に対する基本的なクエリ操作を提供し、将来のデータベース統合やWASM対応の基盤を構築します。また、ForgeScriptの処理基盤として、高速・堅牢・解析可能な設計を目指します。

## Glossary

- **RINQ**: Rust Integrated Query - 本プロジェクトのクエリエンジン
- **Query Builder**: クエリを構築するためのビルダーパターン実装
- **Type State Pattern**: コンパイル時に状態遷移を保証する型レベルのパターン
- **Zero-Cost Abstraction**: ランタイムオーバーヘッドがない抽象化
- **Method Chaining**: メソッドを連鎖的に呼び出すAPI設計
- **Predicate**: フィルタリング条件を表す関数
- **Projection**: データ変換を表す関数
- **Iterator Adapter**: Rustの標準Iteratorトレイトを拡張するアダプター
- **Lazy Evaluation**: 実際に値が必要になるまで評価を遅延させる戦略

## Requirements

### Requirement 1

**User Story:** Rust開発者として、流暢なAPIを使ってコレクションをフィルタリングしたい。そうすることで、読みやすく保守しやすいクエリコードを書くことができる。

#### Acceptance Criteria

1. WHEN 開発者がコレクションを引数に`from()`を呼び出す THEN システムはクエリ可能なオブジェクトを返す
2. WHEN 開発者が述語を引数に`where_()`を呼び出す THEN システムは述語に一致する要素をフィルタリングする
3. WHEN 開発者が複数の`where_()`呼び出しを連鎖させる THEN システムはすべての述語を順番に適用する
4. WHEN 開発者が`collect()`を呼び出す THEN システムはフィルタリングされた結果をVecとして具体化する
5. THE システムは元のコレクションを変更せずに保持する

### Requirement 2

**User Story:** Rust開発者として、コレクション要素を変換したい。そうすることで、データを異なる形状に射影できる。

#### Acceptance Criteria

1. WHEN 開発者が射影関数を引数に`select()`を呼び出す THEN システムは各要素を変換する
2. WHEN 開発者が`where_()`と`select()`を連鎖させる THEN システムは最初にフィルタリングし、次に変換する
3. WHEN 射影関数が異なる型を返す THEN システムは型安全性を保持する
4. THE システムは要素の型を変更する射影をサポートする
5. THE システムは`collect()`が呼び出されるまで射影を遅延評価する

### Requirement 3

**User Story:** Rust開発者として、クエリ結果をソートしたい。そうすることで、出力の順序を制御できる。

#### Acceptance Criteria

1. WHEN 開発者がキーセレクタを引数に`order_by()`を呼び出す THEN システムは要素を昇順でソートする
2. WHEN 開発者が`order_by_descending()`を呼び出す THEN システムは要素を降順でソートする
3. WHEN 開発者が`order_by()`の後に`then_by()`を呼び出す THEN システムは二次ソートを適用する
4. THE システムは複数のキーによるソートをサポートする
5. THE システムは等しい要素に対して安定ソート順序を保持する

### Requirement 4

**User Story:** Rust開発者として、クエリ結果を制限したい。そうすることで、ページネーションとパフォーマンス最適化を実装できる。

#### Acceptance Criteria

1. WHEN 開発者が`take(n)`を呼び出す THEN システムは最大n個の要素を返す
2. WHEN 開発者が`skip(n)`を呼び出す THEN システムは最初のn個の要素をスキップする
3. WHEN 開発者が`skip()`と`take()`を連鎖させる THEN システムはページネーションを正しく実装する
4. WHEN nがコレクションサイズを超える THEN システムはエラーなしで利用可能なすべての要素を返す
5. THE システムは`take()`と`skip()`を遅延評価する

### Requirement 5

**User Story:** Rust開発者として、クエリ結果を集約したい。そうすることで、統計とサマリーを計算できる。

#### Acceptance Criteria

1. WHEN 開発者が`count()`を呼び出す THEN システムは要素の数を返す
2. WHEN 開発者が`first()`を呼び出す THEN システムは最初の要素またはNoneを返す
3. WHEN 開発者が`last()`を呼び出す THEN システムは最後の要素またはNoneを返す
4. WHEN 開発者が述語を引数に`any()`を呼び出す THEN システムはいずれかの要素が一致する場合trueを返す
5. WHEN 開発者が述語を引数に`all()`を呼び出す THEN システムはすべての要素が一致する場合trueを返す

### Requirement 6

**User Story:** Rust開発者として、コンパイル時のクエリ検証が欲しい。そうすることで、エラーを早期に発見できる。

#### Acceptance Criteria

1. WHEN 開発者が無効なクエリを書く THEN システムはコンパイル時エラーを生成する
2. WHEN 開発者がメソッドを間違った順序で呼び出す THEN システムはコンパイルを防ぐ
3. WHEN 開発者が互換性のない型を使用する THEN システムは明確なエラーメッセージを生成する
4. THE システムは型状態パターンを使用して有効なクエリ構築を強制する
5. THE システムは一般的なミスに対して役立つエラーメッセージを提供する

### Requirement 7

**User Story:** Rust開発者として、ゼロコスト抽象化が欲しい。そうすることで、RINQクエリが手書きループと同等のパフォーマンスを発揮する。

#### Acceptance Criteria

1. WHEN RINQクエリがコンパイルされる THEN システムは手動反復と同等のコードを生成する
2. WHEN 手書きループと比較してベンチマークされる THEN システムは測定可能なオーバーヘッドを示さない
3. THE システムはすべてのクエリ操作にインラインヒントを使用する
4. THE システムはRustのイテレータ融合最適化を活用する
5. THE システムはクエリ実行中に不要な割り当てを避ける

### Requirement 8

**User Story:** Rust開発者として、借用データを扱いたい。そうすることで、不要なクローンを避けることができる。

#### Acceptance Criteria

1. WHEN 開発者が借用データをクエリする THEN システムは所有権の移転を要求しない
2. WHEN 開発者が述語で参照を使用する THEN システムはライフタイムを正しく処理する
3. THE システムは所有コレクションと借用コレクションの両方をサポートする
4. THE システムは環境から借用する述語を許可する
5. THE システムは一般的なケースでライフタイム注釈なしでコンパイルする

### Requirement 9

**User Story:** Rust開発者として、明確なエラーメッセージが欲しい。そうすることで、クエリの問題を迅速に修正できる。

#### Acceptance Criteria

1. WHEN クエリが実行時に失敗する THEN システムは説明的なエラーメッセージを提供する
2. WHEN 型の不一致が発生する THEN システムは期待される型と実際の型を示す
3. WHEN コレクションが空で`first()`が呼び出される THEN システムは優雅にNoneを返す
4. THE システムは失敗する可能性のある操作にRustのResult型を使用する
5. THE システムはどのクエリ操作が失敗したかについてのコンテキストを提供する

### Requirement 10

**User Story:** ライブラリメンテナとして、包括的なドキュメントが欲しい。そうすることで、開発者がRINQを簡単に学習できる。

#### Acceptance Criteria

1. WHEN 開発者がAPIドキュメントを表示する THEN システムは各メソッドの例を提供する
2. WHEN 開発者がREADMEを読む THEN システムはクイックスタートガイドを含む
3. THE システムは各操作のパフォーマンス特性を文書化する
4. THE システムは標準イテレータからの移行ガイドを提供する
5. THE システムは一般的な使用パターンとベストプラクティスを含む

### Requirement 11

**User Story:** ForgeScript開発者として、RINQをrusted-caアーキテクチャと統合したい。そうすることで、処理基盤として使用できる。

#### Acceptance Criteria

1. WHEN RINQがrusted-caで使用される THEN システムはクリーンアーキテクチャの原則に従う
2. WHEN RINQクエリが実行される THEN システムはメトリクスコレクタと統合する
3. THE システムは既存のエラーハンドリングパターン（DomainResult、ApplicationResult）をサポートする
4. THE システムはDIコンテナ経由で注入可能である
5. THE システムはテスト可能性のためのトレイトベースの抽象化を提供する

### Requirement 12

**User Story:** 開発者として、RINQクエリをデバッグしたい。そうすることで、クエリ実行を理解できる。

#### Acceptance Criteria

1. WHEN 開発者がデバッグモードを有効にする THEN システムはクエリ操作をログに記録する
2. WHEN 開発者が`inspect()`を呼び出す THEN システムはクエリを消費せずに副作用の観察を許可する
3. THE システムはクエリ実行計画を視覚化する方法を提供する
4. THE システムはクエリ結果のカスタムデバッグフォーマッタをサポートする
5. THE システムはRustの標準デバッグツールと統合する
