# Unison風構文設計

## 概要

x言語にUnison言語の優れた構文要素を取り入れた、最小限のAST設計です。

## 核心的な設計思想

1. **最小AST（3種類のみ）**
   - `Atom`: リテラルと識別子
   - `List`: 適用と複合式
   - `Ann`: 型注釈

2. **Unison風の構文**
   - パイプライン演算子 `|>`
   - ラムダ式 `->`
   - 型注釈 `:`
   - エフェクト `{Effect}`

## 構文例

### 基本的な値定義

```x
-- 値定義
answer = 42

-- 型注釈付き
answer : Int
answer = 42
```

### 関数定義

```x
-- 単純な関数
add : Int -> Int -> Int
add x y = x + y

-- ラムダ式
inc : Int -> Int
inc = x -> x + 1

-- カリー化
multiply : Int -> Int -> Int
multiply = x -> y -> x * y
```

### パイプライン演算子

```x
-- データフロー
result =
  list
    |> map (x -> x * 2)
    |> filter (x -> x > 10)
    |> reduce 0 (+)
```

### エフェクト型

```x
-- IOエフェクト
print : Text ->{IO} ()

-- 複数エフェクト
distributed : List a ->{Remote, Async} List a

-- 純粋関数（エフェクトなし）
pure : Int -> Int
```

## 内部表現

### 1. パース結果（最小AST）

```x
add x y = x + y
```

↓

```
List[
  Atom(Symbol("add")),
  Atom(Symbol("x")),
  Atom(Symbol("y")),
  Atom(Operator("=")),
  List[
    Atom(Symbol("+")),
    Atom(Symbol("x")),
    Atom(Symbol("y"))
  ]
]
```

### 2. ラムダ式の内部表現

```x
x -> x + 1
```

↓

```
List[
  Atom(Symbol("x")),
  Atom(Operator("->")),
  List[
    Atom(Symbol("+")),
    Atom(Symbol("x")),
    Atom(Int(1))
  ]
]
```

### 3. パイプラインの内部表現

```x
list |> reverse |> head
```

↓

```
List[
  List[
    Atom(Symbol("list")),
    Atom(Operator("|>")),
    Atom(Symbol("reverse"))
  ],
  Atom(Operator("|>")),
  Atom(Symbol("head"))
]
```

## 型システム

### 基本型

- `Int` - 整数（デフォルト: i32）
- `Float` - 浮動小数点
- `Text` - 文字列
- `Bool` - 真偽値

### 型コンストラクタ

- `List a` - リスト
- `Maybe a` - オプション
- `Either a b` - 結果型

### 関数型

```x
-- 単純な関数
Int -> Int

-- 多引数（カリー化）
Int -> Int -> Int

-- エフェクト付き
String ->{IO} ()

-- 高階関数
(a -> b) -> List a -> List b
```

## インデントルール

```x
-- ブロックはインデントで表現
processData data =
  let filtered = filter isValid data
      mapped = map transform filtered
  in
    reduce combine initial mapped

-- パイプラインのインデント
result =
  data
    |> step1
    |> step2
    |> step3
```

## 将来の拡張

### パターンマッチング

```x
match list with
  [] -> 0
  x :: xs -> x + sum xs
```

### 代数的データ型

```x
type Maybe a = None | Some a
type Either e a = Left e | Right a
```

### do記法

```x
-- 効果的な計算の連鎖
readAndProcess : () ->{IO} Int
readAndProcess = do
  line <- readLine
  let n = parseInt line
  print ("You entered: " ++ show n)
  return n

-- エラー処理
safeOperation : Int ->{Error} Int
safeOperation x = do
  if x < 0
    then throw "Negative number"
    else return (x * 2)
```

### エフェクトハンドラ

```x
handle
  computation
with Store s ->
  | get k -> k s s
  | put s' k -> k () s'
```

## 実装状況

- ✅ 最小AST定義
- ✅ 基本的なレキサー
- ✅ パーサー（基本機能）
- ⏳ インデントベースのブロック
- ⏳ パターンマッチング
- ⏳ 型推論との統合
- ⏳ エフェクトシステム

## 利点

1. **極めてシンプル** - 3種類のASTノードのみ
2. **表現力豊か** - パイプラインとラムダで複雑な処理を記述
3. **型安全** - 完全な型推論とエフェクト追跡
4. **AIフレンドリー** - 木構造を直接操作しやすい
5. **人間にも読みやすい** - Unisonの優れた構文を採用