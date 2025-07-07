# X-Lang Namespace Management - 使用例

## 概要

x-langのGit風名前空間管理システムの実際の使用例を示します。

## インタラクティブシェルの起動

```bash
$ x namespace shell
x-lang namespace shell v0.1.0
Type 'help' for available commands

/> 
```

## 基本的な操作例

### 1. 関数の作成と編集

```bash
/> edit add
# エディタが開き、以下を入力:
add : Int -> Int -> Int
add x y = x + y

# 保存して終了すると自動コミット
Committed add#a1b2c3d4

/> ls
add#a1b2c3d4
```

### 2. 名前空間の作成と移動

```bash
/> mkdir Core
Created namespace: Core

/> cd Core
/Core> edit id
# エディタで入力:
id : a -> a
id x = x

Committed id#b2c3d4e5

/Core> ls
id#b2c3d4e5

/Core> cd ..
/> ls
add#a1b2c3d4
Core/
```

### 3. バージョン管理

```bash
/> edit add
# 関数を更新:
add : Int -> Int -> Int
add x y = x + y  # optimized version

Committed add#c3d4e5f6

/> log add
c3d4e5f6 - 2024-01-07 15:30:00 - user - Edit add
a1b2c3d4 - 2024-01-07 15:00:00 - user - Create add

/> show add#a1b2c3d4
add : Int -> Int -> Int
add x y = x + y
```

### 4. ファイルシステムとの連携

```bash
# 名前空間をファイルシステムにエクスポート
/> export ./my-project
Exported to ./my-project

# ファイルシステムの内容:
# ./my-project/
#   ├── add.x
#   └── Core/
#       └── id.x

# 別のプロジェクトからインポート
/> import ./other-project
Imported from ./other-project
```

## 高度な使用例

### 1. 複雑な名前空間構造

```bash
/> mkdir Core
/> cd Core
/Core> mkdir List
/Core> cd List
/Core/List> edit map
# map関数を定義

/Core/List> edit filter
# filter関数を定義

/Core/List> ls
map#d4e5f6a7
filter#e5f6a7b8

/Core/List> pwd
/Core/List
```

### 2. 依存関係の確認

```bash
/Core/List> deps map
Dependency analysis not yet implemented
```

### 3. 一括操作

CLIから直接コマンドを実行することも可能：

```bash
# 現在のパスを確認
$ x namespace pwd
/

# 関数一覧を表示
$ x namespace ls
add#a1b2c3d4
Core/

# 関数の内容を表示
$ x namespace cat add
add : Int -> Int -> Int
add x y = x + y

# 特定バージョンを表示
$ x namespace show add#a1b2c3d4
add : Int -> Int -> Int
add x y = x + y

# 履歴を表示
$ x namespace log add
c3d4e5f6 - 2024-01-07 15:30:00 - user - Edit add
a1b2c3d4 - 2024-01-07 15:00:00 - user - Create add
```

## 実装の特徴

1. **自動バージョン管理**: すべての変更が自動的にコミットされる
2. **コンテンツアドレッシング**: SHA-256ハッシュによる一意な識別
3. **シェルライクなインターフェース**: 慣れ親しんだコマンド体系
4. **ファイルシステムとの相互運用**: エクスポート/インポート機能

## 今後の拡張予定

- 依存関係の自動解析
- ブランチ機能
- リモートリポジトリとの同期
- 集中編集モード（関連ファイルの一括編集）