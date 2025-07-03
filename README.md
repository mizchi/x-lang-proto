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

## 参考

- [difftastic](https://github.com/Wilfred/difftastic) - 構文認識型diff
- [Unison Language](https://www.unison-lang.org/) - Content-addressed programming
- Git difftools - カスタムdiffツールの統合