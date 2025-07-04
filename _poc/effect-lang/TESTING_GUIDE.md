# WebAssembly Component Model テストガイド

EffectLangのWebAssembly Component Model機能の包括的なテストスイートです。

## テストの概要

### 1. WIT生成テスト (`tests/wit_generation_tests.rs`)
WebAssembly Interface Types (WIT) ファイルの生成機能をテストします。

**実行方法:**
```bash
cargo test wit_generation_tests
```

**テスト内容:**
- 空のコンパイル単位の処理
- シンプルなインターフェース生成
- 複雑なリソース付きインターフェース
- WASMタイプ変換
- 複数インターフェース処理
- 型定義の生成
- インポート/エクスポート処理
- Visibility修飾子の処理
- 複数戻り値の関数シグネチャ
- 静的メソッド付きリソース
- エラーハンドリング
- 大規模インターフェース
- エッジケース処理

### 2. Component統合テスト (`tests/component_integration_tests.rs`)
WebAssembly Component全体の生成とバックエンド統合をテストします。

**実行方法:**
```bash
cargo test component_integration_tests
```

**テスト内容:**
- 完全なコンポーネント生成
- WITのみ生成
- バックエンドファクトリー統合
- 機能サポート確認
- Rustコード生成詳細
- Cargo.toml生成
- ビルドスクリプト生成
- ランタイム生成
- 型変換
- Visibility処理
- 診断機能
- コード検証
- ファイル出力

### 3. エラーハンドリングテスト (`tests/error_handling_tests.rs`)
様々なエラー条件と例外ケースでの動作をテストします。

**実行方法:**
```bash
cargo test error_handling_tests
```

**テスト内容:**
- WITジェネレーターのエラー処理
- パッケージ名欠如の処理
- 空インターフェースアイテム
- 無効なWASMタイプ
- コンポーネントバックエンドのエラー処理
- WITバックエンドのエラー処理
- 無効なターゲット指定
- 破損したAST構造
- 循環参照
- 特殊文字とUnicode
- メモリ枯渇シミュレーション
- WIT検証エッジケース
- 並行生成
- リソース回復
- 不正なVisibility

### 4. パフォーマンステスト (`tests/performance_tests.rs`)
大規模な入力に対するパフォーマンスとスケーラビリティをテストします。

**実行方法:**
```bash
cargo test performance_tests
```

**ストレステスト実行:**
```bash
cargo test performance_tests -- --ignored
```

**テスト内容:**
- 小規模WIT生成パフォーマンス
- 中規模WIT生成パフォーマンス  
- 大規模WIT生成パフォーマンス
- コンポーネントバックエンドパフォーマンス
- WITバックエンドパフォーマンス
- メモリ使用安定性
- インクリメンタルパフォーマンス
- 深いネスト構造パフォーマンス
- 並行生成パフォーマンス
- 文字列構築パフォーマンス
- 検証パフォーマンス
- Rust生成パフォーマンス
- ストレステスト

### 5. ランタイム実行テスト (`tests/runtime_execution_tests.rs`)
実際のWASMツールとの統合とランタイム実行をテストします。

**実行方法:**
```bash
cargo test runtime_execution_tests
```

**外部ツール必要なテスト実行:**
```bash
cargo test runtime_execution_tests -- --ignored
```

**テスト内容:**
- wasm-toolsでのWITファイル検証
- cargo-componentでのコンポーネントコンパイル
- 生成Rustコードの構文確認
- WIT構文検証
- Component Model準拠性
- wasmtimeコンポーネント検証
- エンドツーエンドコンポーネント生成
- クロスプラットフォームファイル生成

## 必要なツール

### 基本テスト
```bash
# Rustツールチェーン
cargo --version
```

### 外部ツール統合テスト
```bash
# WebAssembly Component Model ツール
cargo install wasm-tools
cargo install cargo-component
cargo install wit-bindgen-cli

# WASMランタイム
cargo install wasmtime-cli
```

## テスト実行コマンド

### 全テスト実行
```bash
cargo test
```

### 特定のテストモジュール実行
```bash
cargo test wit_generation_tests
cargo test component_integration_tests  
cargo test error_handling_tests
cargo test performance_tests
cargo test runtime_execution_tests
```

### 外部ツール必要なテストを含む実行
```bash
cargo test -- --ignored
```

### 詳細出力付き実行
```bash
cargo test -- --nocapture
```

### 並行実行数制限
```bash
cargo test -- --test-threads=1
```

### パフォーマンステストのみ
```bash
cargo test performance_tests
cargo test performance_tests::test_stress_generation -- --ignored
```

## テスト環境セットアップ

### 1. 開発環境
```bash
# Rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WebAssembly ターゲット
rustup target add wasm32-wasi

# Component Model ツール
cargo install wasm-tools cargo-component wit-bindgen-cli wasmtime-cli
```

### 2. CI環境
```bash
# GitHub Actions / CI での例
- name: Install WebAssembly tools
  run: |
    cargo install wasm-tools
    cargo install cargo-component
    rustup target add wasm32-wasi

- name: Run tests
  run: |
    cargo test
    cargo test -- --ignored  # 外部ツール必要なテスト
```

### 3. Docker環境
```dockerfile
FROM rust:1.75

RUN rustup target add wasm32-wasi
RUN cargo install wasm-tools cargo-component wit-bindgen-cli wasmtime-cli

WORKDIR /app
COPY . .
RUN cargo test
RUN cargo test -- --ignored
```

## テスト結果の確認

### 成功例
```
test wit_generation_tests::test_simple_interface_generation ... ok
test component_integration_tests::test_complete_component_generation ... ok
test error_handling_tests::test_wit_generator_error_handling ... ok
test performance_tests::test_wit_generation_performance_small ... ok
test runtime_execution_tests::test_generated_rust_syntax ... ok
```

### エラー例
```
test runtime_execution_tests::test_wit_file_validation_with_wasm_tools ... ignored
# Reason: wasm-tools not available

test performance_tests::test_stress_generation ... ignored  
# Reason: Run only for stress testing
```

## テストデータ

### 生成されるテストファイル
- `./test_output/` - 基本的なテスト出力
- `./perf_test_output/` - パフォーマンステスト出力
- `/tmp/effect_lang_*` - 一時テストファイル

### クリーンアップ
```bash
# テスト成果物をクリーンアップ
rm -rf ./test_output ./perf_test_output
rm -rf /tmp/effect_lang_*
```

## トラブルシューティング

### よくある問題

1. **外部ツールが見つからない**
   ```
   Skipping test: wasm-tools not available
   ```
   **解決:** `cargo install wasm-tools` でツールをインストール

2. **メモリ不足エラー**
   ```
   thread 'test' panicked at 'allocation failed'
   ```
   **解決:** `--test-threads=1` で並行数を制限

3. **一時ディレクトリエラー**
   ```
   Failed to create test directory
   ```
   **解決:** 一時ディレクトリの権限を確認

4. **WIT検証エラー**
   ```
   WIT validation failed: parse error
   ```
   **解決:** 生成されたWITファイルの構文を確認

### デバッグ方法

```bash
# 詳細ログ出力
RUST_LOG=debug cargo test

# 特定のテストのみ詳細実行
cargo test test_name -- --nocapture

# テスト生成物を保持
cargo test --no-run && ls -la test_output/
```

## ベンチマーク

### パフォーマンス基準
- 小規模WIT生成: < 100ms
- 中規模WIT生成: < 1s  
- 大規模WIT生成: < 10s
- コンポーネント生成: < 2s
- 検証処理: < 50ms

### パフォーマンス計測
```bash
# 時間計測付きテスト
time cargo test performance_tests

# メモリ使用量計測
valgrind --tool=massif cargo test performance_tests
```

## 継続的インテグレーション

### GitHub Actions設定例
```yaml
name: WebAssembly Component Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-wasi
      
      - name: Install WASM tools
        run: |
          cargo install wasm-tools
          cargo install cargo-component
      
      - name: Run basic tests
        run: cargo test
      
      - name: Run integration tests
        run: cargo test -- --ignored
```

このテストスイートにより、WebAssembly Component Model機能の品質と信頼性を確保できます。