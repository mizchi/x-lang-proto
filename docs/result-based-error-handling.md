# Result型ベースのエラーハンドリング

## 概要

x言語では例外（throw/catch）ではなく、`Result<Ok, Err>`型を基本としたエラーハンドリングを採用します。これにより、エラーが型システムで明示的に追跡され、より安全なコードが書けます。

## 基本的な型定義

```x
type Result<a, e> = {
  Ok(a);
  Err(e)
};

-- 便利なエイリアス
type alias Result<a> = Result<a, Text>  # デフォルトエラーはText
```

## 基本的な使用方法

### 1. エラーを返す関数

```x
# 除算（ゼロ除算の可能性あり）
safeDivide : Float -> Float -> Result<Float, Text>
safeDivide x y = {
  if y == 0.0 {
    Err("Division by zero")
  } else {
    Ok(x / y)
  }
}
```

### 2. Result型の処理

```x
# パターンマッチング
calculate : Float -> Float -> Text
calculate x y = {
  match safeDivide(x, y) {
    Ok(result) -> { "Result: " ++ show(result) }
    Err(msg) -> { "Error: " ++ msg }
  }
}
```

## Result型のコンビネータ

### map - 成功値を変換

```x
map : (a -> b) -> Result<a, e> -> Result<b, e>
map f result = {
  match result {
    Ok(x) -> { Ok(f(x)) }
    Err(e) -> { Err(e) }
  }
}
```

### flatMap (bind) - 連鎖的な処理

```x
flatMap : (a -> Result<b, e>) -> Result<a, e> -> Result<b, e>
flatMap f result = {
  match result {
    Ok(x) -> { f(x) }
    Err(e) -> { Err(e) }
  }
}

# 演算子版
(>>=) : Result<a, e> -> (a -> Result<b, e>) -> Result<b, e>
(>>=) = { result f -> flatMap(f, result) }
```

### mapError - エラーを変換

```x
mapError : (e -> f) -> Result<a, e> -> Result<a, f>
mapError f result = {
  match result {
    Ok(x) -> { Ok(x) }
    Err(e) -> { Err(f(e)) }
  }
}
```

### withDefault - デフォルト値

```x
withDefault : a -> Result<a, e> -> a
withDefault default result = {
  match result {
    Ok(x) -> { x }
    Err(_) -> { default }
  }
}
```

## do記法での使用

```x
# ResultモナドのためのResultエフェクト
effect ResultE<e> {
  fail : e -> a
}

# Result用のdo記法
parseAndCalculate : Text -> Result<Float, Text>
parseAndCalculate input = with resultHandler do {
  # parseIntがResult<Int, Text>を返す場合
  x <- parseInt(input)
  
  # 条件付きエラー
  if x < 0 {
    fail("Negative numbers not allowed")
  }
  
  # さらなる計算
  y <- safeDivide(100.0, Float(x))
  
  return (y * 2.0)
}

# ハンドラ実装
handler resultHandler {
  fail(e) -> { Err(e) }
  return x -> { Ok(x) }
}
```

## エラー型の設計

### 型安全なエラー

```x
# アプリケーション固有のエラー型
type AppError = {
  NotFound(Text)
  ValidationError(List Text)
  DatabaseError(Text)
  NetworkError(Int, Text)  # status code, message
  InternalError(Text)
}

# 関数の戻り値
findUser : UserId -> Result<User, AppError>
findUser id = {
  match database.query("SELECT * FROM users WHERE id = ?", [id]) {
    [] -> { Err(NotFound("User not found")) }
    [user] -> { Ok(user) }
    _ -> { Err(InternalError("Multiple users with same ID")) }
  }
}
```

## 実用的なパターン

### 1. 早期リターン

```x
processRequest : Request -> Result<Response, AppError>
processRequest req = with resultHandler do {
  # 認証チェック
  user <- authenticate(req.headers) 
    |> mapError(_ -> { Unauthorized("Invalid token") })
  
  # 権限チェック
  if not(hasPermission(user, req.resource)) {
    fail(Forbidden("Access denied"))
  }
  
  # リソース取得
  resource <- findResource(req.resourceId)
  
  # 処理実行
  result <- processResource(resource, req.body)
  
  return Response(200, toJson(result))
}
```

### 2. エラーの集約

```x
# 複数のエラーを集める
type Validation<a> = Result<a, List Text>

validateUser : UserInput -> Validation<ValidUser>
validateUser input = {
  let errors = { [] }
  
  # 各フィールドをチェック
  let errors = {
    if isEmpty(input.name) {
      errors ++ ["Name is required"]
    } else { errors }
  }
  
  let errors = {
    if not(isValidEmail(input.email)) {
      errors ++ ["Invalid email format"]
    } else { errors }
  }
  
  let errors = {
    if length(input.password) < 8 {
      errors ++ ["Password must be at least 8 characters"]
    } else { errors }
  }
  
  # 結果を返す
  if isEmpty(errors) {
    Ok(ValidUser {
      name = input.name
      email = input.email
      passwordHash = hash(input.password)
    })
  } else {
    Err(errors)
  }
}
```

### 3. Result型の合成

```x
# 複数のResult値を合成
sequence : List (Result<a, e>) -> Result<List a, e>
sequence results = {
  results |> fold(Ok([]), (acc, result) -> {
    match (acc, result) {
      (Ok(xs), Ok(x)) -> { Ok(xs ++ [x]) }
      (Err(e), _) -> { Err(e) }
      (_, Err(e)) -> { Err(e) }
    }
  })
}

# 並列バリデーション
validateAll : List (a -> Result<b, e>) -> a -> Result<List b, List e>
validateAll validators input = {
  let results = { validators |> map(v -> { v(input) }) }
  
  let (oks, errs) = { 
    results |> partition(r -> {
      match r {
        Ok(_) -> { true }
        Err(_) -> { false }
      }
    })
  }
  
  if isEmpty(errs) {
    Ok(oks |> map(r -> { match r { Ok(x) -> x } }))
  } else {
    Err(errs |> map(r -> { match r { Err(e) -> e } }))
  }
}
```

## 他のエフェクトとの組み合わせ

```x
# IO + Result
readConfig : Text -> {IO} Result<Config, Text>
readConfig path = with io do {
  exists <- fileExists(path)
  if not(exists) {
    return Err("Config file not found: " ++ path)
  }
  
  content <- readFile(path)
  match parseJson(content) {
    Ok(json) -> { 
      match validateConfig(json) {
        Ok(config) -> { return Ok(config) }
        Err(e) -> { return Err("Invalid config: " ++ e) }
      }
    }
    Err(e) -> { return Err("Parse error: " ++ e) }
  }
}
```

## 利点

1. **型安全**: エラーが型システムで追跡される
2. **明示的**: 関数のシグネチャでエラーの可能性が分かる
3. **合成可能**: コンビネータで簡単に組み合わせ可能
4. **網羅的**: パターンマッチングで全ケースを処理
5. **予測可能**: 例外のような隠れた制御フローがない

## 移行ガイド

### Before (throw/catch)
```x
divide : Float -> Float -> Float
divide x y = {
  if y == 0.0 {
    throw("Division by zero")
  } else {
    x / y
  }
}
```

### After (Result)
```x
divide : Float -> Float -> Result<Float, Text>
divide x y = {
  if y == 0.0 {
    Err("Division by zero")
  } else {
    Ok(x / y)
  }
}
```

この設計により、エラー処理がより予測可能で、保守しやすくなります。