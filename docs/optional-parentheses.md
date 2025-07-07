# 関数呼び出しの括弧省略

## 概要

x言語では、関数呼び出しの括弧 `()` を省略できます。これにより、より簡潔で読みやすいコードが書けます。

## 基本ルール

### 1. 単一引数の関数呼び出し

```x
# 括弧あり
print("Hello")
sqrt(16)
not(true)

# 括弧なし（推奨）
print "Hello"
sqrt 16
not true
```

### 2. 複数引数の関数呼び出し

```x
# 括弧あり
add(2, 3)
substring("hello", 0, 2)

# 括弧なし
add 2 3
substring "hello" 0 2
```

### 3. ネストした関数呼び出し

```x
# 括弧あり（明示的）
print(show(add(2, 3)))

# 括弧なし
print (show (add 2 3))

# パイプライン演算子を使う（推奨）
add 2 3 |> show |> print
```

## 優先順位と結合性

### 関数適用の優先順位

関数適用は最も高い優先順位を持ちます：

```x
# これは (f x) + y と解釈される
f x + y

# これは f (x + y) と解釈される
f (x + y)

# 明確にするために括弧を使用
(f x) + y  # 関数呼び出しの結果に加算
f (x + y)  # 加算の結果を関数に渡す
```

### 左結合性

関数適用は左結合です：

```x
# これは (f x) y と解釈される（カリー化された関数）
f x y

# 3つの引数を持つ関数
g x y z  # ((g x) y) z と同じ
```

## 使用例

### 1. 基本的な使用

```x
# 数学関数
abs -5         # abs(-5)
sqrt 16        # sqrt(16)
max 10 20      # max(10, 20)

# 文字列操作
length "hello"           # length("hello")
concat "Hello" " World"  # concat("Hello", " World")
```

### 2. 条件式での使用

```x
# if式の条件
if not isEmpty list {
  process list
}

# パターンマッチ
match parseJson input {
  Ok json -> process json
  Err msg -> print ("Error: " ++ msg)
}
```

### 3. do記法での使用

```x
readFile : Text ->{IO} Text

processFile : Text ->{IO, Error} Unit
processFile filename = do {
  content <- readFile filename     # readFile(filename)
  lines <- split '\n' content      # split('\n', content)
  
  if isEmpty lines {
    throw "Empty file"
  }
  
  map processLine lines |> sequence
}
```

### 4. with構文での使用

```x
calculate : Int -> Int
calculate n = with state 0 {      # state(0)
  put 10                          # put(10)
  x <- get                        # get()
  put (x + n)                     # put(x + n)
  get                             # get()
}
```

### 5. 高階関数での使用

```x
# map, filter, fold
numbers = [1, 2, 3, 4, 5]

# 括弧なし
result = numbers
  |> filter (x -> x > 2)
  |> map (x -> x * 2)
  |> fold 0 (+)

# 部分適用
add10 = add 10       # add(10)
double = mul 2       # mul(2)

# 関数合成
f = compose square double    # compose(square, double)
```

## ガイドライン

### いつ括弧を使うべきか

1. **複雑な式**: 優先順位が不明確な場合

```x
# 曖昧
f x * y + z

# 明確
(f x) * y + z
f (x * y + z)
```

2. **タプルやレコード**: 構造体を渡す場合

```x
# タプル
process (x, y)      # タプルを渡す
process x y         # 2つの引数を渡す

# レコード
createUser { name = "Alice", age = 30 }
```

3. **ラムダ式を引数にする場合**: 明確性のため

```x
# 括弧あり（推奨）
map (\x -> x * 2) list

# 括弧なし（読みにくい）
map \x -> x * 2 list
```

### スタイルガイド

1. **シンプルな関数呼び出し**: 括弧を省略

```x
print "Hello"
length str
not flag
```

2. **演算子との組み合わせ**: 括弧で明確に

```x
# Good
(length str) > 10
(sqrt x) + (sqrt y)

# Bad (曖昧)
length str > 10
sqrt x + sqrt y
```

3. **パイプライン**: 積極的に使用

```x
# Good
input
  |> trim
  |> toLowerCase
  |> split ' '
  |> length

# Less readable
length (split ' ' (toLowerCase (trim input)))
```

## パーサーの実装

括弧なしの関数呼び出しをサポートするには、パーサーで以下の規則を実装します：

```
# 関数適用（最高優先順位）
application := atom (atom)*

# atom（基本要素）
atom := identifier
      | literal
      | "(" expression ")"
      | "{" block "}"
```

## まとめ

- 関数呼び出しの括弧は省略可能
- 関数適用は最高優先順位を持つ
- 左結合で評価される
- 明確性が必要な場合は括弧を使用
- パイプライン演算子 `|>` との組み合わせが効果的