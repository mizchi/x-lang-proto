# Extensible Effects の例

# 1. 基本的なエフェクト定義
effect State s {
  get : () -> s
  put : s -> ()
}

effect Error e {
  throw : e -> a
  catch : (() -> a) -> (e -> a) -> a
}

effect Logger {
  log : Text -> ()
  debug : Text -> ()
}

# 2. with-first 構文
simpleState : Int
simpleState = with state(0) {
  x <- get()
  put(x + 10)
  get()
}

# 3. 複数ハンドラの合成
multiEffect : Either Text Int
multiEffect = with error with state(0) with logger {
  log("Starting computation")
  x <- get()
  
  if x < 0 then
    throw("Negative state")
  
  debug("State is valid: " ++ show(x))
  put(x + 42)
  
  result <- get()
  log("Result: " ++ show(result))
  return result
}

# 4. エフェクトの拡張
effect TransactionalState s extends State s {
  checkpoint : () -> ()
  rollback : () -> ()
  commit : () -> ()
}

# 拡張されたハンドラ
handler transactional s {
  # State operations
  get() -> resume(s, s)
  put(s') -> resume((), s')
  
  # Transaction operations  
  checkpoints : Ref (List s) = ref []
  
  checkpoint() -> {
    checkpoints := s :: !checkpoints
    resume(())
  }
  
  rollback() -> {
    match !checkpoints {
      [] -> resume(())
      saved :: rest -> {
        checkpoints := rest
        resume((), saved)  # 状態を復元
      }
    }
  }
  
  commit() -> {
    checkpoints := []
    resume(())
  }
}

# 5. Row Polymorphism
genericOperation : {e: Logger | r} => Text ->{e} ()
genericOperation msg = {
  log("Generic: " ++ msg)
  # 他のエフェクトも使える
}

# 6. エフェクトエイリアス
effect alias FileSystem = {Read, Write, Delete}

effect Read {
  readFile : Text -> Text
}

effect Write {
  writeFile : Text -> Text -> ()
}

effect Delete {
  deleteFile : Text -> ()
}

# 7. 条件付きエフェクト
conditionalEffect : Bool -> (() ->{State Int | e} Int) ->{e} Int
conditionalEffect useCache action = 
  if useCache then
    with state(0) handle action()
  else
    action()  # エフェクトをそのまま伝播

# 8. First-class ハンドラ
type Handler e a = (() ->{e} a) -> a

stateHandler : Int -> Handler (State Int) a
stateHandler initial = \action ->
  with state(initial) handle action()

errorHandler : Handler Error (Either Text a)
errorHandler = \action ->
  with error handle action()

# 9. エフェクトの合成パターン
effect Concurrent {
  fork : (() -> a) -> Thread a
  join : Thread a -> a
  parallel : List (() -> a) -> List a
}

effect Distributed extends Concurrent {
  remote : Node -> (() -> a) -> a
  broadcast : List Node -> (() -> ()) -> ()
}

# 10. 実用的な例: Webアプリケーション
effect HTTP {
  get : Text -> Response
  post : Text -> Body -> Response
}

effect Session {
  getSession : () -> SessionData
  setSession : SessionData -> ()
}

effect Database {
  query : Text -> List Row
  execute : Text -> ()
  transaction : (() ->{Database} a) -> a
}

# Webハンドラ
webHandler : Request ->{HTTP, Session, Database, Logger} Response
webHandler req = with logger {
  log("Request: " ++ show(req.path))
  
  session <- getSession()
  
  if not session.authenticated then
    return Response(401, "Unauthorized")
  
  # データベースクエリ
  users <- with database.transaction {
    query("SELECT * FROM users WHERE active = true")
  }
  
  log("Found " ++ show(length(users)) ++ " users")
  return Response(200, toJson(users))
}

# 11. テスト用モックハンドラ
handler mockDatabase {
  testData = [
    Row("id" -> 1, "name" -> "Alice"),
    Row("id" -> 2, "name" -> "Bob")
  ]
  
  query(sql) -> {
    log("Mock query: " ++ sql)
    resume(testData)
  }
  
  execute(sql) -> {
    log("Mock execute: " ++ sql)
    resume(())
  }
  
  transaction(action) -> action()
}

# テスト実行
testWebHandler : () -> ()
testWebHandler = {
  req = Request("GET", "/users", [])
  
  response = with mockDatabase with mockSession with mockLogger {
    webHandler(req)
  }
  
  assert(response.status == 200)
}

# 12. エフェクトのスコープ制御
localEffect : () ->{IO} ()
localEffect = {
  # 外側のIOエフェクト
  print("Outer IO")
  
  # 内側で異なるハンドラを使用
  result = with pureIO {
    print("This is captured")  # 実際には出力されない
    return "done"
  }
  
  # 外側のIOに戻る
  print("Back to outer IO: " ++ result)
}