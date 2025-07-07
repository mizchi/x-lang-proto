# do記法の設計

## 概要

x言語のdo記法は、効果的な計算を順次実行するための構文糖衣です。Haskellのdo記法に似ていますが、代数的エフェクトシステムと統合されています。

## 基本構文

### シンプルなdo記法

```x
printThreeThings : () ->{IO} ()
printThreeThings = do
  print "First"
  print "Second"
  print "Third"
```

### バインド構文（`<-`）

```x
readAndDouble : () ->{IO} Int
readAndDouble = do
  line <- readLine
  let n = parseInt line
  return (n * 2)
```

## 内部表現

do記法は以下のように展開されます：

```x
-- do記法
do
  x <- action1
  action2
  return x

-- 展開後
action1 >>= (x ->
  action2 >>
  return x)
```

## 特徴

### 1. パターンマッチング

```x
processOption : Maybe Int ->{Error} Int
processOption opt = do
  match opt with
    Some n -> return (n * 2)
    None -> throw "No value"
```

### 2. let束縛

```x
calculate : Int ->{Error} Int
calculate x = do
  let y = x * 2
      z = y + 10
  if z > 100
    then throw "Too large"
    else return z
```

### 3. ネストしたdo

```x
nested : () ->{IO} Int
nested = do
  print "Outer"
  result <- do
    print "Inner"
    return 42
  print ("Result: " ++ show result)
  return result
```

### 4. 複数のエフェクト

```x
multiEffect : Text ->{IO, Error, Async} Data
multiEffect url = do
  print ("Fetching: " ++ url)
  response <- httpGet url
  match response with
    Ok data -> return data
    Err msg -> throw msg
```

## 最小AST表現

do記法は最小ASTでは特殊なListとして表現されます：

```x
do
  x <- expr1
  expr2
```

↓

```
List[
  Atom(Symbol("do")),
  List[
    List[Atom(Symbol("<-")), Atom(Symbol("x")), expr1],
    expr2
  ]
]
```

## エフェクトハンドラとの統合

```x
runStateful : Int -> Int
runStateful initial =
  handle
    do
      x <- get
      put (x + 10)
      y <- get
      return y
  with State initial s ->
    | get k -> k s s
    | put s' k -> k () s'
```

## 構文糖衣の展開規則

### 基本規則

1. `do { e }` → `e`
2. `do { e; stmts }` → `e >> do { stmts }`
3. `do { p <- e; stmts }` → `e >>= (\p -> do { stmts })`
4. `do { let decls; stmts }` → `let decls in do { stmts }`

### 演算子

- `>>=` : バインド演算子
- `>>` : 順次実行演算子（結果を無視）
- `return` : 純粋な値をエフェクト文脈に持ち上げる

## インデント/ブレース両対応

```x
-- インデントスタイル
doSomething = do
  x <- action1
  y <- action2
  return (x + y)

-- ブレーススタイル
doSomething = do {
  x <- action1;
  y <- action2;
  return (x + y)
}

-- 混在
doSomething = do
  x <- action1
  y <- do { action2a; action2b }
  return (x + y)
```

## エラー処理パターン

```x
-- 早期リターン
validateAndProcess : Text ->{Error} Int
validateAndProcess input = do
  n <- parseInt input
  if n < 0
    then throw "Negative not allowed"
    else return (n * 2)

-- try-catch相当（将来実装）
safeOperation : () ->{Error} Int
safeOperation = do
  try
    riskyOperation
  catch
    | "FileNotFound" -> return 0
    | other -> throw other
```

## 実装優先度

1. ✅ 基本的なdo構文
2. ✅ `<-` バインド
3. ✅ let束縛
4. ⏳ パターンマッチング in do
5. ⏳ エフェクトハンドラとの統合
6. ⏳ try-catch構文

do記法により、効果的な計算を命令型スタイルで記述でき、コードの可読性が大幅に向上します。