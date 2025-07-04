# x Language CLI クイックスタート

x Language CLI (`x`) は、AST に対して直接操作を行う x 言語のコマンドラインツールです。

## インストール

### このプロジェクト内で実行する場合

```bash
# プロジェクトディレクトリに移動
cd /home/mizchi/ideas/binary-ast-diff

# x-cliをビルド
cargo build --release --bin x

# バイナリは target/release/x に生成されます
# またはcargoを使って直接実行:
cargo run --bin x -- --help
```

### 開発時の実行方法

```bash
# デバッグビルドで実行（高速ビルド）
cargo run --bin x -- new my-project
cargo run --bin x -- convert input.rustic.x --to binary
cargo run --bin x -- show input.x --format tree

# リリースビルドで実行（最適化済み）
cargo run --release --bin x -- compile input.x --target typescript
```

### プロジェクト構造

```
binary-ast-diff/
├── x-cli/              # CLIツール実装
│   ├── src/
│   │   ├── main.rs     # メインエントリーポイント
│   │   ├── commands/   # 各コマンドの実装
│   │   ├── format.rs   # フォーマット変換
│   │   └── utils.rs    # ユーティリティ
├── x-parser/           # パーサー・AST実装
├── x-editor/           # エディタ機能（未実装）
├── x-checker/          # 型チェッカー（未実装）
├── x-compiler/         # コンパイラ（未実装）
└── docs/               # ドキュメント
```

## 基本コマンド

### 1. 新しいプロジェクトを作成

```bash
x new my-project
```

### 2. フォーマット変換・表示

#### バイナリファイル (.x) の内容を表示
```bash
# Rust ライクな構文で表示（推奨）
cargo run --bin x -- show input.x --format rustic

# OCaml ライクな構文で表示
cargo run --bin x -- show input.x --format ocaml

# JSON 形式で表示
cargo run --bin x -- show input.x --format json
```

#### バイナリ形式からテキストファイルへ変換（必要時のみ）
```bash
cargo run --bin x -- convert input.x --to rustic
cargo run --bin x -- convert input.x --to ocaml  
cargo run --bin x -- convert input.x --to json
```

#### テキスト形式からバイナリ形式へ変換
```bash
cargo run --bin x -- convert input.rustic.x --to binary
cargo run --bin x -- convert input.ocaml.x --to binary
```

#### 自動フォーマット検出
```bash
# 拡張子から自動的にフォーマットを検出
cargo run --bin x -- convert input.rustic.x output.x
```

### 3. AST 構造の詳細表示

#### ツリー形式で表示
```bash
cargo run --bin x -- show input.x --format tree
```

#### 型情報とスパン情報を含めて表示
```bash
cargo run --bin x -- show input.x --format tree --types --spans
```

#### サマリー表示
```bash
cargo run --bin x -- show input.x --format summary
```

#### コンパクト形式で表示
```bash
cargo run --bin x -- show input.x --format compact --depth 5
```

### 4. AST クエリ

#### 型でノードを検索
```bash
cargo run --bin x -- query input.x "type:ValueDef"
cargo run --bin x -- query input.x "type:function"
```

#### シンボルでノードを検索
```bash
cargo run --bin x -- query input.x "symbol:main"
cargo run --bin x -- query input.x "symbol:test"
```

#### 定義を検索
```bash
cargo run --bin x -- query input.x "def:MyFunction"
```

#### JSON 形式で結果を出力
```bash
cargo run --bin x -- query input.x "type:ValueDef" --format json
```

### 5. 型チェック

```bash
cargo run --bin x -- check input.x
cargo run --bin x -- check input.x --detailed  # 詳細な情報を表示
cargo run --bin x -- check input.x --quiet     # エラーのみ表示
```

### 6. コンパイル

```bash
cargo run --bin x -- compile input.x --target typescript
cargo run --bin x -- compile input.x --target wasm --output dist/
```

### 7. インタラクティブ編集

```bash
cargo run --bin x -- edit input.x --interactive
```

### 8. プロジェクト統計

```bash
cargo run --bin x -- stats . --format table
cargo run --bin x -- stats . --format json
```

### 9. REPL

```bash
cargo run --bin x -- repl --syntax rustic
cargo run --bin x -- repl --preload my-module.x
```

### 10. LSP サーバー

```bash
cargo run --bin x -- lsp --mode stdio
cargo run --bin x -- lsp --mode tcp --port 9257
```

## サポートされているフォーマット

### バイナリフォーマット
- `.x` - x Language バイナリフォーマット（マジックナンバー: `\0xlg`）

### テキストフォーマット
- `.rustic.x` - Rust ライクな構文
- `.ocaml.x` - OCaml ライクな構文  
- `.lisp.x` - S-expression 構文
- `.haskell.x` - Haskell ライクな構文
- `.json` - JSON 表現

## 設定

設定ファイルは `~/.config/x-lang/config.toml` に保存されます。

```toml
# デフォルト構文スタイル
default_syntax = "rustic"

# デフォルト出力フォーマット
default_format = "auto"

[editor]
# 型情報をデフォルトで表示
show_types = true
# スパン情報をデフォルトで表示
show_spans = false
# 最大ツリー表示深度
max_depth = 10

[compiler]
# デフォルトターゲット言語
default_target = "typescript"
# 最適化レベル
optimization_level = 2
# ソースマップ生成
source_maps = true

[lsp]
# LSP サーバーポート
port = 9257
# 有効な LSP 機能
features = ["completions", "hover", "goto-definition", "find-references", "rename"]
```

## 例：基本的なワークフロー

### 1. 新しいプロジェクトを作成
```bash
# プロジェクト内でcargoを使って実行
cargo run --bin x -- new hello-world
cd hello-world
```

### 2. バイナリ形式（.x）でコードを直接作成・編集
```bash
# AST を直接操作してバイナリファイルを生成
cargo run --bin x -- edit main.x --interactive

# または既存のテンプレートから生成
cargo run --bin x -- new --template simple main.x
```

### 3. バイナリファイル (.x) を Rust ライクな構文で確認
```bash
# .x ファイルの内容を rustic 形式で表示
cargo run --bin x -- show main.x --format rustic

# または一時的に rustic ファイルとして出力
cargo run --bin x -- convert main.x --to rustic
# → main.rustic.x が生成される
```

### 4. AST 構造を確認
```bash
# AST ツリー構造を表示
cargo run --bin x -- show main.x --format tree --types

# JSON 形式でも確認可能
cargo run --bin x -- show main.x --format json
```

### 5. 型チェック
```bash
cargo run --bin x -- check main.x
```

### 6. TypeScript にコンパイル
```bash
cargo run --bin x -- compile main.x --target typescript --output dist/
```

## デフォルトワークフロー: .x ファイル中心の開発

x Language では、**バイナリ形式の .x ファイルが第一級**です。テキスト形式は主に表示・確認用として使用します。

### バイナリファースト開発の利点
- **ゼロパース時間**: テキスト解析が不要
- **構造保証**: AST の整合性が常に保たれる
- **高速操作**: 直接 AST 操作による高速編集
- **型安全性**: AST レベルでの型情報保持

### 推奨ワークフロー

```bash
# 1. .x ファイルで開発
cargo run --bin x -- edit main.x

# 2. 内容確認（必要に応じてテキスト表示）
cargo run --bin x -- show main.x --format rustic

# 3. 構造確認
cargo run --bin x -- show main.x --format tree

# 4. 型チェック・コンパイル
cargo run --bin x -- check main.x
cargo run --bin x -- compile main.x
```

## 実装状況

### ✅ 完全実装済み
- **CLI基盤**: 全12コマンドの構造
- **フォーマット変換**: バイナリ ↔ テキスト変換（基本実装）
- **AST表示**: tree, JSON, summary, compact形式
- **クエリ機能**: 型・シンボル検索
- **設定管理**: TOML設定ファイル

### 🚧 部分実装済み
- **バイナリシリアライゼーション**: 基本構造のみ（完全なAST → バイナリ変換は未実装）
- **テキストパーサー**: プレースホルダー実装（実際の構文解析は未実装）

### ❌ 未実装
- **x-checker**: 型チェック機能
- **x-compiler**: コンパイル機能  
- **x-editor**: エディタ機能
- **実際のAST操作**: rename, extract等
- **REPL**: インタラクティブ実行環境
- **LSP**: Language Server Protocol

### 開発時の注意事項

```bash
# 現在は基本的なCLI構造とダミー実装のため、
# 実際のファイル処理は限定的です

# 動作確認できるコマンド:
cargo run --bin x -- --help
cargo run --bin x -- new test-project
cargo run --bin x -- show --help

# 未実装機能を実行した場合の例:
cargo run --bin x -- check input.x
# → "Type checking is not yet implemented" と表示される
```

## トラブルシューティング

### バイナリファイルが読めない
```bash
# ファイルの形式を確認
file main.x

# マジックナンバーを確認
hexdump -C main.x | head -1
# 00786c67 (\0xlg) で始まっているはず
```

### パース エラー
```bash
# 詳細なエラー情報を表示
x check input.x --detailed
```

### 設定の確認
```bash
# 設定ファイルの場所を確認
x config --show-path

# デフォルト設定を生成
x config --init
```

## 高度な使用法

### バッチ変換
```bash
# ディレクトリ内のすべての .rustic.x ファイルをバイナリに変換
find . -name "*.rustic.x" -exec cargo run --bin x -- convert {} \;
```

### パイプライン処理
```bash
# AST をJSON で出力して jq で処理
cargo run --bin x -- show input.x --format json | jq '.kind.modules[0].items'
```

### 型情報の抽出
```bash
# すべての関数定義を検索
cargo run --bin x -- query input.x "type:ValueDef" --format json | jq '.[] | select(.node_type == "ValueDef")'
```

## 開発に貢献する

### 新しい機能を実装する場合

1. **パーサー機能** (`x-parser/`)
   ```bash
   # テキスト構文の実装
   vim x-parser/src/syntax/rust_like.rs
   cargo test -p x-parser
   ```

2. **型チェッカー機能** (`x-checker/`)
   ```bash
   # 型推論・制約解決の実装
   vim x-checker/src/lib.rs
   cargo test -p x-checker
   ```

3. **コンパイラ機能** (`x-compiler/`)
   ```bash
   # TypeScript/WASM出力の実装
   vim x-compiler/src/codegen/typescript.rs
   cargo test -p x-compiler
   ```

4. **CLI機能拡張** (`x-cli/`)
   ```bash
   # 新しいコマンドの追加
   vim x-cli/src/commands/new_command.rs
   cargo test -p x-cli
   ```

### テスト実行

```bash
# 全体テスト
cargo test

# 特定クレートのテスト
cargo test -p x-cli
cargo test -p x-parser

# 統合テスト
cargo test --test integration_tests
```

## 参考資料

- [アーキテクチャ設計](../design/ast-architecture.md)
- [実装ガイド](../design/implementation-guide.md)
- [バイナリフォーマット仕様](../design/binary-format.md)

## ヘルプ

各コマンドの詳細なヘルプを表示：

```bash
cargo run --bin x -- --help
cargo run --bin x -- convert --help
cargo run --bin x -- show --help
cargo run --bin x -- query --help
```

## 実際に動作確認してみる

```bash
# 1. プロジェクトに移動
cd /home/mizchi/ideas/binary-ast-diff

# 2. ヘルプを表示
cargo run --bin x -- --help

# 3. 新しいプロジェクトを作成
cargo run --bin x -- new my-test-project

# 4. 変換コマンドのヘルプを確認
cargo run --bin x -- convert --help

# 5. showコマンドのヘルプを確認
cargo run --bin x -- show --help

# 6. queryコマンドのヘルプを確認  
cargo run --bin x -- query --help
```

現在実装されている機能は基本的なCLI構造とプレースホルダー実装ですが、
将来的な拡張に向けた堅牢な基盤が整っています。