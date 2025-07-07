# with構文の設計（Unison + Koka風）

## 概要

x言語の`with`構文は、UnisonとKokaの良さを組み合わせた代数的エフェクトハンドラです。

## 基本構文

### シンプルなwithハンドラ

```x
-- Koka風の簡潔な構文
result = with handler {
  computation
}

-- Unison風の明示的な構文
result = handle computation with handler
```

### インラインハンドラ定義

```x
-- 状態エフェクトの例
statefulComputation : Int
statefulComputation = with state(0) {
  x <- get()
  put(x + 10)
  y <- get()
  return y
}

-- より明示的な形
statefulComputation = with {
  state = 0
  get() = resume(state)
  put(s) = state := s; resume(())
} in {
  x <- get()
  put(x + 10)
  y <- get()
  return y
}
```

## Koka風の拡張構文

### 1. エフェクトの定義

```x
-- エフェクト定義
effect State s {
  get : () -> s
  put : s -> ()
}

effect Error {
  throw : Text -> a
}

effect Async {
  await : Promise a -> a
  sleep : Int -> ()
}
```

### 2. ハンドラの定義

```x
-- 名前付きハンドラ
handler stateHandler s {
  get() -> resume(s, s)
  put(s') -> resume((), s')
  return x -> x
}

-- エラーハンドラ
handler errorHandler {
  throw(msg) -> Left(msg)
  return x -> Right(x)
}
```

### 3. with構文の使用

```x
-- 単一エフェクト
result = with stateHandler(42) {
  x <- get()
  put(x * 2)
  get()
}

-- 複数エフェクトの合成
result = with errorHandler with stateHandler(0) {
  x <- get()
  if x < 0
    then throw("Negative state")
    else put(x + 1)
  return x
}
```

## Unison風の要素

### handle/with構文

```x
-- Unisonスタイル
result = 
  handle
    x <- get()
    put(x + 10)
    get()
  with State 0 ->
    | get() k -> k state state
    | put(s') k -> k () s'
```

### 統合構文

```x
-- Unison + Koka のハイブリッド
result = handle {
  x <- get()
  put(x + 10)
  get()
} with state(0)
```

## 高度な機能

### 1. エフェクトの合成

```x
-- 複数のエフェクトを扱う
processData : List Int ->{State Int, Error} Int
processData items = with state(0) with error {
  for item in items {
    if item < 0
      then throw("Negative item")
      else {
        sum <- get()
        put(sum + item)
      }
  }
  get()
}
```

### 2. ローカルハンドラ

```x
-- 部分的にエフェクトを処理
partialHandling : () ->{IO, Error} Int
partialHandling = {
  -- Error だけローカルに処理
  result = with error {
    data <- readFile("config.txt")
    parseConfig(data)
  }
  
  -- IO エフェクトは外側で処理される
  match result {
    Ok(config) -> return config.value
    Err(msg) -> print(msg); return 0
  }
}
```

### 3. エフェクトの変換

```x
-- あるエフェクトを別のエフェクトに変換
handler asyncToState {
  await(promise) -> {
    put(Loading)
    result = actualAwait(promise)
    put(Loaded(result))
    resume(result)
  }
}
```

## 実装例

### カウンター

```x
counter : () -> Int
counter = with state(0) {
  increment <- λ() { n <- get(); put(n + 1) }
  increment()
  increment()
  increment()
  get()
} -- 結果: 3
```

### エラー処理付き計算

```x
safeCalculation : Int -> Either Text Int
safeCalculation x = with error {
  if x < 0 then throw("Negative input")
  if x > 100 then throw("Too large")
  return (x * x)
}
```

### 非同期処理

```x
fetchAndProcess : Text -> Int
fetchAndProcess url = with async with error {
  response <- httpGet(url)
  data <- parseResponse(response)
  processData(data)
}
```

## do記法との統合

```x
-- with内でdo記法を使用
complexOperation : () -> Int
complexOperation = with state(0) with error do
  x <- get()
  if x < 0 then throw("Invalid state")
  
  -- ネストしたwith
  y <- with state(100) do
    a <- get()
    put(a + x)
    get()
  
  put(x + y)
  return y
```

## 構文糖衣

```x
-- 簡潔な形
with state(0) { ... }

-- 展開後
handle { ... } with State 0 ->
  | get() k -> k state state
  | put(s') k -> k () s'
  | return x -> x
```

## 最小AST表現

```x
with state(0) { get() }
```

↓

```
List[
  Atom(Symbol("with")),
  List[
    Atom(Symbol("state")),
    Atom(Int(0))
  ],
  List[
    Atom(Symbol("get")),
    List[]
  ]
]
```

## 利点

1. **Kokaの簡潔さ**: `with handler { ... }` の直感的な構文
2. **Unisonの柔軟性**: パターンマッチングによる細かい制御
3. **合成可能**: 複数のハンドラを自然に組み合わせ可能
4. **型安全**: エフェクトが型システムで追跡される
5. **段階的**: 必要に応じて部分的にエフェクトを処理

この設計により、UnisonとKokaの両方の良さを活かした、表現力豊かで使いやすいエフェクトシステムが実現できます。