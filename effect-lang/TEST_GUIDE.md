# EffectLang Testing Guide

このガイドでは、EffectLangプロジェクトのテストスイートについて説明します。

## テストの構成

### 単体テスト

#### 1. レキサーテスト (`tests/lexer_tests.rs`)
- トークン化の正確性
- キーワード認識
- 識別子、数値、文字列の処理
- 演算子とコメントの処理
- エラー処理とスパン情報

**実行方法:**
```bash
make test-lexer
# または
cargo test lexer_tests
```

#### 2. パーサーテスト (`tests/parser_tests.rs`)
- AST構築の正確性
- モジュール、関数、型の解析
- パターンマッチングと制御構造
- インポート/エクスポートの処理
- エラー回復機能

**実行方法:**
```bash
make test-parser
# または
cargo test parser_tests
```

#### 3. バイナリシリアライゼーションテスト (`tests/binary_tests.rs`)
- AST→バイナリ→ASTのラウンドトリップ
- データ完全性の検証
- 圧縮効率の測定
- コンテンツハッシュの一貫性
- パフォーマンス特性

**実行方法:**
```bash
make test-binary
# または
cargo test binary_tests
```

#### 4. バイナリASTディフテスト (`tests/diff_tests.rs`)
- 構造的差分検出
- 変更の種類の識別（追加、削除、変更）
- 差分フォーマット
- パフォーマンス最適化
- ハッシュベース最適化

**実行方法:**
```bash
make test-diff
# または
cargo test diff_tests
```

#### 5. シンボルテスト (`tests/symbol_tests.rs`)
- シンボルインターニング
- 事前定義シンボル
- スコープ管理
- 可視性制御
- シンボルテーブル操作

**実行方法:**
```bash
make test-symbols
# または
cargo test symbol_tests
```

### 統合テスト

#### 統合テスト (`tests/integration_tests.rs`)
- CLIコマンドのエンドツーエンドテスト
- コンパイル→解析→差分のワークフロー
- エラー処理とバリデーション
- ファイルI/O操作
- パフォーマンス測定

**実行方法:**
```bash
make test-integration
# または
cargo test integration_tests
```

## パフォーマンステスト

### ベンチマーク (`benches/performance_bench.rs`)
Criterionを使用した詳細なパフォーマンス測定：

- **レキサー**: トークン化速度
- **パーサー**: 解析速度
- **シリアライゼーション**: バイナリ変換速度
- **差分**: 構造比較速度
- **エンドツーエンド**: 全パイプライン性能

**実行方法:**
```bash
make bench
# または
cargo bench

# 特定のベンチマーク
make bench-lexer
make bench-parser
make bench-diff
```

### パフォーマンステスト
```bash
make perf
```
以下をテスト：
- 大規模ASTの処理性能
- 圧縮効率
- 差分計算速度

## テスト実行方法

### 基本実行
```bash
# 全テスト実行
make test
# または
cargo test

# 詳細出力付き
make test-verbose
```

### 個別テスト実行
```bash
# 特定のテストファイル
cargo test lexer_tests
cargo test parser_tests
cargo test binary_tests
cargo test diff_tests
cargo test symbol_tests
cargo test integration_tests

# 特定のテスト関数
cargo test test_simple_module_roundtrip
cargo test test_identical_modules_diff
```

### 開発サイクル
```bash
# クイック開発サイクル
make dev
# フォーマット → 型チェック → テスト（詳細出力）
```

### CI/CD チェック
```bash
make ci
# フォーマット → リント → 型チェック → テスト
```

## テストデータ

### 例ファイル生成
```bash
make examples
```
これにより以下が実行されます：
1. サンプルファイル生成
2. コンパイルテスト
3. 差分テスト
4. 解析テスト

### カスタムテストファイル
```bash
# 手動でテストファイルを作成
echo 'module Custom
let test = fun x -> x * 2' > custom.eff

# コンパイルテスト
./target/release/effect-cli compile -i custom.eff --timing

# 解析
./target/release/effect-cli analyze custom.eff.bin --hash --size
```

## テスト設定

### 環境変数
```bash
# デバッグログ有効化
RUST_LOG=debug cargo test

# バックトレース有効化
RUST_BACKTRACE=1 cargo test
```

### テストフィルタリング
```bash
# 名前でフィルタ
cargo test roundtrip

# 成功したテストを除外
cargo test -- --nocapture

# 並列実行を無効化
cargo test -- --test-threads=1
```

## パフォーマンス分析

### メモリ使用量
```bash
make memcheck
# Valgrindを使用したメモリ解析
```

### プロファイリング
```bash
# perf使用（Linux）
perf record --call-graph=dwarf cargo test test_large_ast_performance
perf report

# Instrumentsの使用（macOS）
cargo instruments -t "Time Profiler" cargo test test_large_ast_performance
```

### ベンチマーク結果の分析
```bash
cargo bench
# target/criterion/フォルダにHTMLレポート生成
open target/criterion/report/index.html
```

## 継続的インテグレーション

### GitHubアクション設定例
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: make ci
      - run: make examples
      - run: make bench
```

## トラブルシューティング

### よくある問題

1. **コンパイルエラー**
   ```bash
   cargo check
   ```

2. **テスト失敗**
   ```bash
   cargo test -- --nocapture
   ```

3. **パフォーマンス問題**
   ```bash
   cargo bench
   make perf
   ```

4. **メモリリーク**
   ```bash
   make memcheck
   ```

### デバッグ方法
```bash
# 特定のテストをデバッグモードで実行
cargo test test_function_name -- --nocapture

# GDBでのデバッグ
rust-gdb target/debug/deps/test_binary-*
```

## カバレッジ測定

```bash
# cargo-tarpaulinのインストール
cargo install cargo-tarpaulin

# カバレッジ測定
make coverage
# または
cargo tarpaulin --out Html
```

## ベストプラクティス

1. **テスト書き方**
   - 1つのテスト関数で1つの機能をテスト
   - 分かりやすいテスト名を使用
   - エラーケースもテスト

2. **パフォーマンステスト**
   - 現実的なデータサイズでテスト
   - 回帰を検出できる閾値設定
   - メモリ使用量も監視

3. **統合テスト**
   - 実際のユーザーワークフローをシミュレート
   - エラー処理の確認
   - ファイルシステム操作の検証

このテストスイートにより、EffectLangの信頼性と性能を継続的に保証できます。