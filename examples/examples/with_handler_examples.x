# with構文の例（ブレースファースト）

# 1. 基本的なStateエフェクト
simpleCounter : Int;
simpleCounter = with state(0) {
  put(10);
  x <- get();
  put(x + 5);
  get()
}; # 結果: 15

# 2. エラーハンドリング
safeDivision : Int -> Int -> Either Text Int;
safeDivision x y = with error {
  if y == 0 {
    throw("Division by zero")
  } else {
    return (x / y)
  }
};

# 3. 複数エフェクトの合成
validateAndCompute : Int -> Either Text Int;
validateAndCompute input = with error with state(input) {
  x <- get();
  if x < 0 { throw("Negative input") };
  if x > 100 { throw("Too large") };
  
  put(x * 2);
  result <- get();
  return result
};

# 4. カスタムエフェクト定義
effect Logger {
  log : Text -> ();
  debug : Text -> ()
};

# 5. カスタムハンドラ
handler consoleLogger {
  log(msg) -> { print("[LOG] " ++ msg); resume(()) };
  debug(msg) -> { print("[DEBUG] " ++ msg); resume(()) };
  return x -> { x }
};

# 6. ロギング付き計算
computeWithLogging : Int -> Int;
computeWithLogging n = with consoleLogger {
  log("Starting computation");
  debug("Input: " ++ show(n));
  
  let result = { n * n + n };
  
  log("Computation complete");
  return result
};

# 7. ネストしたwith
nestedHandlers : () -> Int
nestedHandlers = with state(0) {
  put(10)
  
  # 内側で別の状態を使用
  inner = with state(100) {
    x <- get()
    put(x + 1)
    get()
  }
  
  outer <- get()
  return (outer + inner) # 10 + 101 = 111
}

# 8. エフェクトの変換
effect Async {
  await : Promise a -> a
}

handler asyncToError {
  await(promise) -> 
    match waitFor(promise) {
      Success(value) -> resume(value)
      Failure(err) -> throw("Async failed: " ++ err)
    }
}

# 9. リソース管理
withResource : Text -> (Handle ->{IO, Error} a) -> Either Text a
withResource path action = with error {
  handle <- openFile(path)
  try {
    result = action(handle)
    closeFile(handle)
    return result
  } finally {
    closeFile(handle)
  }
}

# 10. イテレータ/ジェネレータ風
effect Yield a {
  yield : a -> ()
}

generateNumbers : () ->{Yield Int} ()
generateNumbers = {
  yield(1)
  yield(2)
  yield(3)
}

collectYielded : (() ->{Yield a} ()) -> List a
collectYielded gen = with state([]) {
  handler yielder {
    yield(x) -> {
      xs <- get()
      put(xs ++ [x])
      resume(())
    }
    return _ -> get()
  }
  
  with yielder {
    gen()
  }
}

# 11. トランザクション風
effect Transaction {
  read : Key -> Value
  write : Key -> Value -> ()
  abort : Text -> a
}

transactionalUpdate : Key -> (Value -> Value) -> Either Text ()
transactionalUpdate key f = with transaction {
  original <- read(key)
  newValue = f(original)
  
  if isValid(newValue)
    then write(key, newValue)
    else abort("Invalid value")
}

# 12. 非決定性計算
effect Choice {
  choose : List a -> a
}

findSolution : () ->{Choice} (Int, Int)
findSolution = {
  x <- choose([1, 2, 3, 4, 5])
  y <- choose([1, 2, 3, 4, 5])
  
  if x + y == 7
    then return (x, y)
    else choose([]) # バックトラック
}

# 13. Continuation風
effect Control {
  shift : ((a -> b) -> b) -> a
}

# 14. パイプラインとの組み合わせ
processData : List Int -> Int
processData data = with state(0) {
  data
    |> filter (x -> x > 0)
    |> map (x -> { log("Processing: " ++ show x); x * 2 })
    |> fold (acc x -> {
         sum <- get()
         put(sum + x)
         return (acc + x)
       }) 0
}

# 15. do記法との統合
complexWorkflow : Text -> Either Text Result
complexWorkflow input = with error with state(emptyState) do
  # 検証フェーズ
  validated <- validate(input)
  
  # 処理フェーズ
  result <- with logger do
    log("Processing: " ++ validated)
    process(validated)
  
  # 保存フェーズ
  saved <- with transaction do
    write("result", result)
    write("timestamp", now())
    return result
  
  return saved