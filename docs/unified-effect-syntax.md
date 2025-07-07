# 統合エフェクト構文設計

## 概要

x言語は、UnisonとKokaの良いところを統合したエフェクトシステムを提供します。`with`構文を中心に、`do`記法と自然に組み合わせられる設計です。

## 構文の統一

### 基本形

```x
-- Koka風（推奨）
result = with handler {
  computation
}

-- Unison風（互換性のため）
result = handle computation with handler

-- 両者は完全に等価
```

### with構文の3つの形式

#### 1. 組み込みハンドラ

```x
-- 状態エフェクト
with state(42) {
  x <- get()
  put(x + 1)
  get()
}

-- エラーエフェクト
with error {
  if condition
    then throw("Error message")
    else return value
}
```

#### 2. 名前付きハンドラ

```x
handler myStateHandler init {
  get() -> resume(state, state)
  put(s) -> resume((), s)
  return x -> (x, state)  -- 最終状態も返す
}

with myStateHandler(0) {
  -- 使用
}
```

#### 3. インラインハンドラ

```x
with {
  get() -> resume(currentState)
  put(s) -> currentState := s; resume(())
} in {
  -- 計算
}
```

## エフェクトの定義

```x
-- エフェクト定義
effect State s {
  get : () -> s
  put : s -> ()
}

effect Error e {
  throw : e -> a
  catch : (() -> a) -> (e -> a) -> a
}

effect IO {
  print : Text -> ()
  readLine : () -> Text
}

effect Async {
  await : Promise a -> a
  async : (() -> a) -> Promise a
}
```

## with と do の組み合わせ

### 基本パターン

```x
-- with の中で do を使用
result = with state(0) do
  x <- get()
  put(x + 10)
  y <- get()
  return y

-- ネストした with/do
complexCalc = with error do
  x <- with state(100) do
    a <- get()
    put(a * 2)
    get()
  
  if x > 200
    then throw("Too large")
    else return x
```

### 簡潔な記法

```x
-- 単一の式なら波括弧を省略可能
simpleGet = with state(42) get()

-- パイプラインとの組み合わせ
result = with state(0) 
  list |> map incrementAndCount |> sum
  
where incrementAndCount x = do
  count <- get()
  put(count + 1)
  return (x + 1)
```

## 実用的なパターン

### 1. リソース管理

```x
withFile : Text -> (Handle ->{IO, Error} a) -> a
withFile path action = with {
  resource = openFile(path)
  return x -> { closeFile(resource); x }
  throw e -> { closeFile(resource); throw e }
} in action(resource)

-- 使用例
content = withFile "data.txt" \handle -> do
  readAll(handle)
```

### 2. トランザクション

```x
atomically : (() ->{STM} a) -> a
atomically action = with stm {
  try {
    result = action()
    commit()
    return result
  } catch {
    rollback()
    retry()
  }
}
```

### 3. 並行処理

```x
parallel : List (() ->{Async} a) -> List a
parallel tasks = with async {
  promises = tasks |> map async
  promises |> map await
}
```

### 4. ロギング付き計算

```x
withLogging : Text -> (() ->{Logger} a) -> a
withLogging prefix action = with {
  log(msg) -> {
    print("[" ++ prefix ++ "] " ++ msg)
    resume(())
  }
} in action()
```

## 高度な機能

### エフェクトの合成

```x
-- 複数のエフェクトを扱う
processRequest : Request ->{IO, Error, State Session} Response
processRequest req = do
  session <- get()
  
  if not (isValid session)
    then throw(InvalidSession)
  
  response <- with async do
    data <- fetchData(req.url)
    process(data)
  
  put(updateSession session)
  return response
```

### エフェクトの変換

```x
-- Promise を Either に変換
promiseToEither : Promise a -> Either Text a
promiseToEither promise = with {
  await(p) -> 
    match waitFor(p) {
      Success(x) -> resume(Right(x))
      Failure(e) -> resume(Left(e))
    }
} in await(promise)
```

### 部分的なハンドリング

```x
-- Error だけを処理し、IO はそのまま
partialHandle : () ->{IO, Error} Int
partialHandle = do
  result <- with error {
    line <- readLine()  -- IO エフェクトは通過
    parseInt(line)      -- Error はここで処理
  }
  
  match result {
    Left(err) -> print("Error: " ++ err); return 0
    Right(n) -> return n
  }
```

## 構文糖衣の展開

```x
-- 糖衣構文
with state(0) do
  x <- get()
  put(x + 1)
  get()

-- 展開後
handle {
  x <- get()
  put(x + 1)
  get()
} with State 0 s ->
  | get() k -> k s s
  | put(s') k -> k () s'
  | return x -> x
```

## 設計原則

1. **簡潔性**: Kokaの`with`構文を基本とする
2. **表現力**: Unisonのパターンマッチングを活用
3. **組み合わせ**: `do`記法と自然に統合
4. **段階的**: 必要に応じて詳細度を選択可能
5. **型安全**: すべてのエフェクトが型で追跡される

この設計により、簡潔で表現力豊かなエフェクトシステムが実現できます。