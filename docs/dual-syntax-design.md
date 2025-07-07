# インデント/ブロック両対応構文設計

## 概要

x言語は、インデントベースと`{}`/`;`ブロックベースの両方の構文をサポートします。これらは完全に等価で、自由に混在させることができます。

## 基本原則

1. **インデントブロック** = **`{}`ブロック**
2. **改行** = **`;`**（文の区切り）
3. **同じインデント** = **同じブロック内**

## 構文の等価性

### 1. 関数定義

```x
-- インデントベース
add x y =
  let sum = x + y
  sum

-- ブロックベース（完全に等価）
add x y = {
  let sum = x + y;
  sum
}

-- 1行で書くことも可能
add x y = { let sum = x + y; sum }
```

### 2. パイプライン

```x
-- インデントベース
result =
  data
    |> filter positive
    |> map double
    |> reduce 0 (+)

-- ブロックベース
result = {
  data
    |> filter positive
    |> map double
    |> reduce 0 (+)
}

-- セミコロンで区切ることも可能
result = { data |> filter positive; |> map double; |> reduce 0 (+) }
```

### 3. 条件分岐

```x
-- インデントベース
abs x =
  if x < 0
    then -x
    else x

-- ブロックベース
abs x = {
  if x < 0 {
    then -x
  } else {
    x
  }
}

-- 混在も可能
abs x =
  if x < 0 { then -x }
  else x
```

### 4. Let式

```x
-- インデントベース
calculate x y =
  let a = x * 2
      b = y * 3
  in
    a + b

-- ブロックベース
calculate x y = {
  let a = x * 2;
      b = y * 3;
  in
    a + b
}

-- より明示的なブロック
calculate x y = {
  let {
    a = x * 2;
    b = y * 3;
  };
  in a + b
}
```

### 5. パターンマッチ

```x
-- インデントベース
factorial n =
  match n with
    0 -> 1
    n -> n * factorial (n - 1)

-- ブロックベース
factorial n = {
  match n with {
    0 -> 1;
    n -> n * factorial (n - 1)
  }
}
```

## レキサーの拡張

### トークン追加

```rust
enum Token {
    // 既存のトークン...
    
    // ブロック用トークン
    LeftBrace,      // {
    RightBrace,     // }
    Semicolon,      // ;
    
    // レイアウトトークン（内部用）
    BlockStart,     // 仮想的なブロック開始
    BlockEnd,       // 仮想的なブロック終了
}
```

### レイアウト変換

```rust
// インデントベースのコードを内部的に変換
add x y =
  x + y

// ↓ 内部表現では以下と等価

add x y = BlockStart
  x + y
BlockEnd
```

## パーサーの設計

### ブロック認識

```rust
fn parse_block(&mut self) -> Result<Vec<Expr>> {
    match self.peek() {
        Some(Token::LeftBrace) => {
            // 明示的なブロック
            self.advance(); // {
            let exprs = self.parse_block_items()?;
            self.expect(Token::RightBrace)?;
            Ok(exprs)
        }
        Some(Token::Indent(_)) => {
            // インデントブロック
            self.parse_indent_block()
        }
        _ => {
            // 単一式
            Ok(vec![self.parse_expression()?])
        }
    }
}
```

## 混在例

```x
-- 関数定義はインデント、内部はブロック
processData data =
  let filtered = filter isValid data in {
    filtered
      |> map transform
      |> reduce combine initial;
  }

-- トップレベルはブロック、内部はインデント
processData data = {
  let filtered = filter isValid data;
  in
    filtered
      |> map transform
      |> reduce combine initial
}
```

## スタイルガイドライン

### 推奨: インデントベース

```x
-- Clean and readable
module DataProcessor

process : List a -> Result b
process data =
  data
    |> validate
    |> transform
    |> finalize
```

### 推奨: 1行の場合はブロック

```x
-- Short functions
inc x = { x + 1 }
double x = { x * 2 }
```

### 推奨: 混在は最小限に

```x
-- OK: 明確な境界で使い分け
handleRequest req =
  let response = { processRequest req } in
  sendResponse response

-- 避ける: 不規則な混在
handleRequest req = {
  let response =
    processRequest req  -- インデント？
  ; sendResponse response }
```

## 実装上の利点

1. **段階的移行** - 既存コードを徐々に変換可能
2. **ツール対応** - エディタが苦手な構文を回避
3. **柔軟性** - 状況に応じて最適な表現を選択
4. **互換性** - 他言語からの移植が容易

## 変換ツール

```bash
# インデント形式に統一
x fmt --style=indent file.x

# ブロック形式に統一
x fmt --style=block file.x

# 混在を許可（デフォルト）
x fmt --style=mixed file.x
```

## まとめ

この二重構文サポートにより：
- Pythonユーザーはインデントスタイルで
- C/Java/JavaScriptユーザーはブロックスタイルで
- 同じ言語を自然に書くことができます

内部的には同じASTに変換されるため、どちらの形式で書いても実行結果は完全に同じです。