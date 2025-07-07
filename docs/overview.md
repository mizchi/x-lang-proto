# x Language プロジェクト概要

x Language は、Unison 言語にインスパイアされた、コンテンツアドレス指向の関数型プログラミング言語です。関数の内容に基づいてハッシュを生成し、バージョン管理を行うことで、依存関係の管理と並行バージョンの共存を実現します。

## 主要な特徴

### 1. コンテンツアドレス指向
- 関数やデータ型の内容からハッシュを生成
- 同じ内容は同じハッシュを持つため、重複を排除
- ハッシュによる確実な参照

### 2. 暗黙的バージョニング
- 関数は自動的にバージョンが付与される
- セマンティックバージョニング（major.minor.patch）をサポート
- 型シグネチャの変更を検知して互換性をチェック

### 3. 関数レベルの依存管理
- 各関数が独自の依存関係を持つ
- インポート時にバージョン指定が可能（例：`import Math@"^1.0.0"`）
- 並行バージョンの共存が可能

### 4. エフェクトシステム
- 代数的エフェクトによる副作用の管理
- エフェクトハンドラによる柔軟な制御フロー
- 型安全な副作用の合成

## プロジェクト構造

### コアクレート

#### x-parser
AST（抽象構文木）の定義とパーサーの実装。

主要機能：
- Haskell風構文のサポート（`fun`と`fn`の両方でラムダ式を記述可能）
- S式構文のサポート（実験的）
- バイナリシリアライゼーション
- コンテンツハッシュの計算
- 依存関係の抽出
- メタデータ管理

重要なモジュール：
- `ast.rs` - AST定義
- `parser.rs` - パーサー実装
- `content_hash.rs` - 決定論的ハッシュ計算
- `versioning.rs` - バージョン管理
- `signature.rs` - 関数シグネチャの抽出
- `metadata.rs` - 人間可読な名前とハッシュのマッピング

#### x-checker
型チェッカーと型推論エンジン。

主要機能：
- Hindley-Milner型推論
- 代数的データ型
- エフェクト型の推論
- パターンマッチングの網羅性チェック
- レコード型とバリアント型
- 型エイリアス

#### x-compiler
複数のバックエンドへのコンパイラ。

サポートするターゲット：
- WebAssembly（Component ModelとGC提案）
- TypeScript（ESModules、CommonJS）
- JavaScript
- WIT（WebAssembly Interface Types）

#### x-editor
AST編集とコンテンツアドレス管理。

主要機能：
- 永続的ASTデータ構造
- インクリメンタルな編集操作
- 名前空間管理
- コンテンツリポジトリ
- ツリー類似度アルゴリズム（APTED、TSED）

#### x-cli
コマンドラインインターフェース。

実装されたコマンド：
- `x parse` - ソースコードのパース
- `x check` - 型チェック
- `x compile` - コンパイル
- `x run` - 実行
- `x hash` - コンテンツハッシュの表示
- `x version` - バージョン管理
  - `show` - 関数のバージョン情報表示
  - `tag` - バージョンタグの付与
  - `check` - バージョン互換性チェック
  - `deps` - 依存関係の表示
- `x imports` - インポート情報の表示
- `x outdated` - 古い依存関係の検出

### 補助クレート

#### x-ast-builder
プログラム的にASTを構築するためのビルダーAPI。

#### x-ai-codegen
AIを使用したコード生成（実験的）。

#### x-testing
テストランナーとテストディスカバリー。

## 実装された機能

### 1. 構文とパース
```x
module Math

-- 関数定義（funとfnの両方をサポート）
let add = fun x y -> x + y
let multiply = fn x y -> x * y

-- パターンマッチング
let factorial = fn n ->
  match n with
  | 0 -> 1
  | n -> n * factorial (n - 1)

-- エフェクトハンドラ
effect State s where
  get : () -> s
  put : s -> ()

let stateful = handle
  let x = perform get () in
  perform put (x + 1);
  perform get ()
with State s ->
  | get () resume -> resume s s
  | put s' resume -> resume () s'
```

### 2. バージョン管理
```x
-- バージョン指定付きインポート
import Math@"^1.0.0" (add, multiply)
import DataStructures@"~2.1.0" (List, Map)

-- 関数は自動的にバージョンが付与される
-- 内容が変わるとハッシュも変わる
let processData = fn data ->
  List.map (fn x -> x * 2) data
```

### 3. コンテンツハッシュ
```bash
$ x hash myfile.x
add#a3f5c2b1: fun x y -> x + y
multiply#d7e9f3a2: fn x y -> x * y
```

### 4. 依存関係管理
```bash
$ x outdated myproject.x
Checking for outdated dependencies...

Found 2 outdated dependencies:

Math.sqrt:
  Current: v1.0.0
  Latest: v1.2.0
  Used by: calculateDistance

DataStructures.Map.insert:
  Current: v2.0.0
  Latest: v2.1.3
  Used by: processUserData, cacheResults
```

### 5. バージョン互換性チェック
```bash
$ x version check Math.add v1.0.0 v1.1.0
Checking compatibility between v1.0.0 and v1.1.0...
✓ Compatible: Signatures match
  Both versions: (Int, Int) -> Int
```

## 設計原則

### 1. イミュータビリティ
- すべてのデータ構造は不変
- 関数は純粋で副作用を持たない（エフェクトシステムで管理）

### 2. コンテンツアドレッシング
- 内容によって一意に識別
- ロックファイル不要（Unisonの哲学）
- 確実な再現性

### 3. 人間にやさしいインターフェース
- 読みやすい構文（Haskell風）
- 明確なエラーメッセージ
- 段階的な型注釈

### 4. 実用性
- WebAssemblyやTypeScriptへのコンパイル
- 既存のエコシステムとの統合
- エディタサポート

## 今後の開発予定

1. **名前解決システム** - 人間可読な名前からハッシュへの解決
2. **コードベースブラウザ** - Unison UCM風のインタラクティブな環境
3. **リファクタリングツール** - ハッシュ更新の自動化
4. **バージョンマイグレーション** - 依存関係の一括更新
5. **差分表示** - バージョン間の変更内容の可視化
6. **名前空間の再設計** - より柔軟なモジュールシステム
7. **エフェクトシステムの改善** - より表現力豊かなエフェクト合成

## 技術スタック

- **言語**: Rust
- **パーサー**: 手書きの再帰下降パーサー
- **型システム**: Hindley-Milner + 拡張（エフェクト、レコード、バリアント）
- **永続データ構造**: `im` クレート
- **シリアライゼーション**: カスタムバイナリフォーマット
- **ハッシュ**: SHA-256ベースの決定論的ハッシュ

## インスピレーション

このプロジェクトは以下の言語やプロジェクトから影響を受けています：

- **Unison** - コンテンツアドレッシングとコードベース管理
- **Haskell** - 純粋関数型プログラミングと型システム
- **OCaml** - 代数的データ型とパターンマッチング
- **Eff/Frank** - 代数的エフェクトシステム
- **Nix** - 再現可能なビルドとコンテンツアドレッシング