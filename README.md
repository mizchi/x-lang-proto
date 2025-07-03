# バイナリAST Diff プロジェクト

S式の構文とバイナリシリアライザーを使ってバイナリASTの構造差分を可視化するdiffツールです。

## 目標

- S式の構文仕様とバイナリシリアライザーの実装
- バイナリASTの構造差分を可視化するdiffツール
- Git difftoolsとの統合
- Unisonスタイルのcontent-addressed codeの考え方を取り入れたバイナリコード表現

## 設計思想

1. **S式ベースのAST表現**: コードをS式として表現し、構造的な差分を計算
2. **バイナリシリアライザー**: S式を効率的なバイナリ形式で保存
3. **Content-Addressed Storage**: Unisonライクなハッシュベースのコード識別
4. **構造的Diff**: テキストではなくAST構造の差分を可視化

## インストール

```bash
# 依存関係のインストール
npm install

# プロジェクトのビルド
npm run build

# Git difftoolsとして統合（オプション）
./git-integration/install.sh
```

## 使用方法

### 基本的なコマンド

#### S式ファイルのパース
```bash
# S式をパースしてASTを表示
npm run dev -- parse examples/example1.sexp

# ハッシュ情報も表示
npm run dev -- parse examples/example1.sexp --hash

# バイナリ表現も表示
npm run dev -- parse examples/example1.sexp --binary
```

#### ファイル間の差分比較
```bash
# 基本的な差分表示
npm run dev -- diff examples/example1.sexp examples/example2.sexp

# 構造的な差分表示
npm run dev -- diff examples/example1.sexp examples/example2.sexp --structural

# コンパクト表示（変更のみ）
npm run dev -- diff examples/example1.sexp examples/example2.sexp --compact

# 色なし表示
npm run dev -- diff examples/example1.sexp examples/example2.sexp --no-color
```

#### バイナリ表現の比較
```bash
# バイナリサイズとハッシュの比較
npm run dev -- binary-diff examples/example1.sexp examples/example2.sexp
```

### Git統合

#### 手動でのGit difftools使用
```bash
# 特定のツールとして使用
git difftool --tool=binary-ast-diff file1.sexp file2.sexp

# ファイルタイプ別の自動適用（.gitattributesに設定後）
git diff examples/example1.sexp
```

#### .gitattributesの設定例
```gitattributes
# S式ファイルに対してカスタムdiffを適用
*.sexp diff=sexp
```

#### Git設定例
```bash
# グローバル設定
git config --global difftool.binary-ast-diff.cmd "binary-ast-diff git-diff \"\$MERGED\" \"\$LOCAL\" \"\$REMOTE\""
git config --global diff.sexp.tool "binary-ast-diff"
```

### 実際の使用例

```bash
# 階乗関数の実装の差分を確認
npm run dev -- diff examples/example1.sexp examples/example2.sexp

# 複雑なモジュールの構造的変更を確認
npm run dev -- diff examples/complex1.sexp examples/complex2.sexp --structural

# Content-addressedハッシュでファイルを識別
npm run dev -- parse examples/example1.sexp --hash
# Output: Content Hash: 10e7e257
```

### S式の書き方

このツールは以下のS式構文をサポートします：

```lisp
; アトム
42
3.14
"文字列"
#t  ; true
#f  ; false

; シンボル
factorial
+
my-function

; リスト
(+ 1 2)
(defun factorial (n)
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

; 複雑な構造
(module math
  (export factorial fibonacci)
  (defun factorial (n) ...)
  (defstruct point (x 0) (y 0)))
```

## 特徴

### Content-Addressed Storage
- Unisonスタイルのハッシュベースコード識別
- SHA256による内容の一意性保証
- 同じ構造は同じハッシュを生成

### 構造的Diff
- テキストレベルではなくAST構造での差分
- Myers algorithmベースの効率的な差分計算
- 階層的な変更の可視化

### バイナリ効率性
- 可変長エンコーディングによる効率的な保存
- バイナリ形式でのS式シリアライゼーション
- ファイルサイズとハッシュの比較

## 参考

- [difftastic](https://github.com/Wilfred/difftastic) - 構文認識型diff
- [Unison Language](https://www.unison-lang.org/) - Content-addressed programming
- Git difftools - カスタムdiffツールの統合