# ブレースファースト構文設計

## 概要

x言語は`{}`を基本とした明示的なブロック構文を採用します。オフサイドルール（インデントベース）は補助的な機能として提供しますが、推奨しません。

## 基本原則

1. **明示性**: すべてのブロックは`{}`で囲む
2. **一貫性**: セミコロン`;`は複数文を同一行に記述する時のみ必須
3. **可読性**: 適切な改行とインデントで整形
4. **非曖昧性**: パーサーにとって明確な構造

## 推奨構文スタイル

### 1. 関数定義

```x
# 推奨: 明示的なブレース
add : Int -> Int -> Int
add x y = { x + y }

# より複雑な関数
factorial : Int -> Int
factorial n = {
  if n == 0 {
    1
  } else {
    n * factorial(n - 1)
  }
}
```

### 2. do記法

```x
# 推奨: do ブロックも明示的に
readAndProcess : () ->{IO, Error} Int
readAndProcess = do {
  line <- readLine()
  n <- parseInt(line)
  print("Got: " ++ show(n))
  return n
}
```

### 3. with構文

```x
# 推奨: with も明示的に
statefulComputation : Int
statefulComputation = with state(0) {
  x <- get()
  put(x + 10)
  get()
}

# 複数ハンドラ
multiEffect : Int
multiEffect = with error with state(0) {
  x <- get()
  if x < 0 { throw("Negative") }
  put(x + 1)
  get()
}
```

### 4. パターンマッチ

```x
# 推奨: match の各ケースも明示的に
processOption : Maybe Int -> Int
processOption opt = {
  match opt {
    None -> { 0 }
    Some(x) -> { x * 2 }
  }
}
```

### 5. let式

```x
# 推奨: let ブロックも明示的に
calculate : Int -> Int
calculate x = {
  let {
    y = x * 2
    z = y + 10
  }
  if z > 100 {
    z / 2
  } else {
    z
  }
}
```

## 構文規則

### ブロックのルール

1. **ブロック開始**: `{` の後に改行（推奨）
2. **ブロック終了**: `}` は独立した行に（推奨）
3. **セミコロン**: 複数文を同一行に記述する場合のみ`;`が必須
   - 通常は行末のセミコロンは不要
   - 1行に複数文を書く場合は区切りとして必要

### セミコロンの規則

```x
# セミコロンが必要な場合
multiStatement = {
  print("First")     # セミコロン不要
  print("Second")    # セミコロン不要
  42                 # セミコロン不要
}

# 同一行に複数文を書く場合
oneLine = { print("First"); print("Second"); 42 }

# do記法内
doBlock = do {
  x <- action1()     # セミコロン不要
  y <- action2()     # セミコロン不要
  return (x + y)     # セミコロン不要
}
```

## 1行記法

短い関数や式は1行で記述可能：

```x
# 簡潔な関数
inc : Int -> Int
inc x = { x + 1 }

# 簡潔なwith
simpleState = with state(0) { get() }

# 簡潔なif
abs x = { if x < 0 { -x } else { x } }
```

## インデントスタイル（補助機能）

オフサイドルールは残しますが、以下の場合のみ使用を推奨：

```x
# REPLやプロトタイピング時のみ
# 本番コードでは非推奨
quickTest =
  let x = 42
      y = 10
  in x + y
```

## モジュール構造

```x
# モジュールも明示的に
module DataStructures {
  
  # エクスポート
  export { List, map, filter }
  
  # 型定義
  type List a = {
    Nil
    Cons(a, List a)
  }
  
  # 関数定義
  map : (a -> b) -> List a -> List b
  map f list = {
    match list {
      Nil -> { Nil }
      Cons(x, xs) -> { Cons(f(x), map(f, xs)) }
    }
  }
  
  filter : (a -> Bool) -> List a -> List a
  filter pred list = {
    match list {
      Nil -> { Nil }
      Cons(x, xs) -> {
        if pred(x) {
          Cons(x, filter(pred, xs))
        } else {
          filter(pred, xs)
        }
      }
    }
  }
}
```

## エフェクト定義

```x
# エフェクトも明示的に
effect State s {
  get : () -> s
  put : s -> ()
}

effect Error {
  throw : Text -> a
  catch : (() -> a) -> (Text -> a) -> a
}
```

## 利点

1. **明確性**: ブロックの境界が視覚的に明確
2. **ツール対応**: エディタやフォーマッタが扱いやすい
3. **エラー回復**: パースエラーからの回復が容易
4. **一貫性**: すべての構造で同じルール
5. **移植性**: 他言語からの移行が容易

## 移行ガイド

### インデントスタイルからの移行

```x
# Before (インデント)
factorial n =
  if n == 0
    then 1
    else n * factorial (n - 1)

# After (ブレース)
factorial n = {
  if n == 0 {
    1
  } else {
    n * factorial(n - 1)
  }
}
```

### フォーマッタ

```bash
# ブレーススタイルに自動変換
x fmt --style=braces input.x

# インデントを保持（非推奨）
x fmt --style=offside input.x
```

## まとめ

- **基本は`{}`**: すべてのブロックを明示的に
- **セミコロン`;`**: 同一行に複数文を書く場合のみ必要
- **インデント**: 読みやすさのためだけに使用
- **一貫性**: プロジェクト全体で統一

この設計により、曖昧性のない、ツールフレンドリーな構文を実現します。