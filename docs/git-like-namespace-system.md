# Git風名前空間管理システム

## 概要

x言語の名前空間をGitのようなバージョン管理システムとして扱い、シェルライクなインターフェースで操作できるシステムを設計します。名前空間をディレクトリ、関数をファイルとして扱い、編集後は自動的にコミットされます。

## 実装状況

Git風名前空間管理システムの基本実装が完了しました。以下のコマンドが利用可能です：

```bash
# CLIからnamespace管理システムを起動
$ x namespace shell

# または個別のコマンドを直接実行
$ x namespace pwd
$ x namespace ls
$ x namespace cat <function>
$ x namespace edit <function>
$ x namespace show <function>#<hash>
$ x namespace log <function>
$ x namespace export <directory>
$ x namespace import <directory>
```

## 基本概念

### 1. 構造のマッピング

```
Git/ファイルシステム → x言語
─────────────────────────────
ディレクトリ        → 名前空間/モジュール
ファイル           → 関数/値定義
コミット           → バージョン（ハッシュ付き）
ブランチ           → 名前空間のバリエーション
```

### 2. 基本コマンド

```bash
# 現在の名前空間を表示
$ pwd
/Core/List

# 名前空間の内容を表示
$ ls
map#a3f4b2c1
filter#b2e5d3a2
fold#c1d4e5f3
sort#d5e2f1a4

# 名前空間を移動
$ cd /Math
$ cd ../String

# 関数の内容を表示
$ cat map
map : (a -> b) -> List a -> List b
map f list = match list {
    Nil -> Nil
    Cons x xs -> Cons (f x) (map f xs)
}

# 関数を編集
$ edit map
# エディタが開く...
# 保存して閉じると自動的にコミット

# 過去のバージョンを表示
$ show map#a3f4b2c1
# 特定のハッシュのバージョンを表示

# 履歴を表示
$ log map
c7d8e9f0 - 2024-01-07 14:30:25 - Add tail recursion optimization
b2e5d3a2 - 2024-01-06 10:15:00 - Fix type annotation
a3f4b2c1 - 2024-01-05 09:00:00 - Initial implementation
```

## 詳細設計

### 1. 名前空間の構造

```x
# 内部表現
type Namespace = {
    path : List String         # e.g., ["Core", "List"]
    entries : Map String Entry # 名前 -> エントリ
    parent : Maybe Namespace
}

type Entry = {
    Function {
        name : String
        versions : List Version
        current : Hash
    }
    Module {
        namespace : Namespace
    }
}

type Version = {
    hash : Hash
    timestamp : DateTime
    author : String
    message : String
    content : FunctionDef
    dependencies : Set Hash
}
```

### 2. シェルインターフェース

```x
# シェルコマンドの実装
effect Shell {
    pwd : () -> Path
    cd : Path -> ()
    ls : () -> List Entry
    cat : String -> String
    edit : String -> ()
    show : String -> Hash -> String
    log : String -> List Version
}

# シェルハンドラ
handler shellHandler state {
    pwd() -> { resume state.currentPath }
    
    cd path -> {
        let newPath = resolvePath state.currentPath path
        resume () with { state | currentPath = newPath }
    }
    
    ls() -> {
        let ns = getNamespace state.currentPath
        resume (listEntries ns)
    }
    
    cat name -> {
        let entry = lookupEntry state.currentPath name
        resume (formatEntry entry)
    }
    
    edit name -> {
        let entry = lookupEntry state.currentPath name
        let newContent = openEditor entry.content
        let newHash = computeHash newContent
        let newVersion = createVersion newContent
        updateEntry state.currentPath name newVersion
        resume ()
    }
}
```

### 3. 自動コミットシステム

```x
# 編集時の自動コミット
editFunction : Path -> String -> {Shell, IO} ()
editFunction path name = do {
    # 現在のバージョンを取得
    current <- getFunction path name
    
    # エディタで編集
    content <- openEditor current.content
    
    # 変更があれば自動コミット
    if content != current.content {
        # ハッシュ計算
        newHash <- computeHash content
        
        # 依存関係解析
        deps <- analyzeDependencies content
        
        # バージョン作成
        version = Version {
            hash = newHash
            timestamp = now()
            author = getCurrentUser()
            message = "Edit " ++ name
            content = content
            dependencies = deps
        }
        
        # コミット
        commitVersion path name version
        
        print ("Committed " ++ name ++ "#" ++ show newHash)
    }
}
```

### 4. 依存関係追跡

```x
# 関数の依存関係を解析
analyzeDependencies : FunctionDef -> Set Hash
analyzeDependencies func = do {
    # ASTを走査して参照を収集
    refs <- collectReferences func.body
    
    # 各参照のハッシュを解決
    refs |> map resolveReference |> Set.fromList
}

# 依存関係グラフ
type DependencyGraph = Map Hash (Set Hash)

# 影響範囲の計算
calculateImpact : Hash -> DependencyGraph -> Set Hash
calculateImpact changed graph = {
    # 変更された関数に依存するすべての関数を収集
    transitiveClosure changed graph
}
```

### 5. ファイルシステムへの展開

```x
# ディレクトリへの展開
exportToFileSystem : Path -> FilePath -> {IO} ()
exportToFileSystem namespacePath targetDir = do {
    ns <- getNamespace namespacePath
    
    # ディレクトリ作成
    createDirectory targetDir
    
    # 各エントリを展開
    ns.entries |> Map.iter \name entry -> {
        match entry {
            Function func -> {
                # .x ファイルとして保存
                let content = formatFunction func
                writeFile (targetDir </> name ++ ".x") content
            }
            Module subNs -> {
                # サブディレクトリとして再帰的に展開
                exportToFileSystem (namespacePath </> name) 
                                 (targetDir </> name)
            }
        }
    }
}

# ファイルシステムからのインポート
importFromFileSystem : FilePath -> Path -> {IO} ()
importFromFileSystem sourceDir namespacePath = do {
    files <- listDirectory sourceDir
    
    files |> List.iter \file -> {
        if endsWith ".x" file {
            # 関数としてインポート
            content <- readFile (sourceDir </> file)
            func <- parseFunction content
            importFunction namespacePath (dropExtension file) func
        } else if isDirectory (sourceDir </> file) {
            # モジュールとして再帰的にインポート
            importFromFileSystem (sourceDir </> file) 
                               (namespacePath </> file)
        }
    }
}
```

### 6. 集中編集モード

```x
# 関連ファイルの集中編集
editRelated : String -> {Shell, IO} ()
editRelated pattern = do {
    # パターンにマッチする関数を検索
    matches <- findFunctions pattern
    
    # 依存関係を解析
    deps <- matches |> List.flatMap \f -> {
        getDependencies f ++ getDependents f
    } |> Set.fromList
    
    # 関連ファイルをすべて表示
    print "Related functions:"
    (matches ++ Set.toList deps) |> List.iter \f -> {
        print ("  " ++ f.path ++ "/" ++ f.name)
    }
    
    # マルチファイル編集セッションを開始
    openMultiEditor (matches ++ Set.toList deps)
}
```

## 使用例

### 1. 基本的なワークフロー

```bash
# プロジェクトのルートに移動
$ cd /MyProject

# モジュール構造を確認
$ ls
Core/
Utils/
Tests/
main#e5f6a7b8

# Core モジュールに移動
$ cd Core

# リスト操作関数を確認
$ ls | grep List
List/

$ cd List
$ ls
map#a1b2c3d4
filter#b2c3d4e5
fold#c3d4e5f6

# map 関数を編集
$ edit map
# エディタが開く...
# 編集後、自動的にコミット

# 変更履歴を確認
$ log map
d7e8f9a0 - Just now - Edit map
a1b2c3d4 - Yesterday - Initial implementation
```

### 2. バージョン管理

```bash
# 特定バージョンの確認
$ show map#a1b2c3d4
map : (a -> b) -> List a -> List b
map f list = match list {
    Nil -> Nil
    Cons x xs -> Cons (f x) (map f xs)
}

# 差分の確認
$ diff map#a1b2c3d4 map#d7e8f9a0
- map f list = match list {
+ map f = List.match {
      Nil -> Nil
-     Cons x xs -> Cons (f x) (map f xs)
+     Cons x xs -> Cons (f x) (map f xs)  # tail recursive
  }
```

### 3. 依存関係の確認

```bash
# 関数の依存関係を表示
$ deps map
Depends on:
  - List.match#b3c4d5e6
  - Cons#c4d5e6f7
  - Nil#d5e6f7a8

Used by:
  - Utils.transform#e6f7a8b9
  - Tests.mapTest#f7a8b9c0
```

### 4. ファイルシステムへの展開

```bash
# 現在の名前空間をディレクトリに展開
$ export ./src
Exported to ./src:
  ./src/map.x
  ./src/filter.x
  ./src/fold.x

# ディレクトリから名前空間にインポート
$ import ./external /External
Imported 15 functions from ./external
```

## 実装の利点

1. **バージョン管理**: すべての変更が自動的に記録される
2. **依存関係追跡**: 変更の影響範囲が明確
3. **効率的な編集**: 関連ファイルを集中的に編集可能
4. **柔軟な表示**: ハッシュで任意のバージョンを参照可能
5. **相互運用性**: ファイルシステムとの相互変換が可能

## 設定とカスタマイズ

```x
# 設定ファイル (.xconfig)
config = {
    editor = "vim"                    # 使用するエディタ
    autoCommit = true                 # 自動コミットの有効/無効
    showHash = "short"                # ハッシュ表示形式 (full/short/none)
    defaultBranch = "main"            # デフォルトブランチ
    hooks = {
        preCommit = ["lint", "test"]  # コミット前フック
        postCommit = ["notify"]        # コミット後フック
    }
}
```

このシステムにより、x言語のコードベースをGitのように管理しながら、言語の特性を活かした効率的な開発が可能になります。