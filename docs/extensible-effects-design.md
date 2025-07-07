# Extensible Effects 設計

## 概要

x言語のextensible effectsは、エフェクトを合成・拡張可能にする仕組みです。`with`を`handle`より前に書く構文で、より自然な記述を可能にします。

## 基本構文

### with-first構文

```x
-- 新しい構文: with が前
result = with state(0) handle {
  x <- get()
  put(x + 10)
  get()
}

-- 短縮形（handleを省略）
result = with state(0) {
  x <- get()
  put(x + 10)
  get()
}
```

### 複数ハンドラの合成

```x
-- 左から右に適用される
result = with logger with state(0) with error {
  log("Starting")
  x <- get()
  if x < 0 then throw("Negative")
  put(x + 1)
  log("Done")
  get()
}
```

## Extensible Effects

### 1. エフェクトの定義

```x
-- 基本的なエフェクト
effect State s {
  get : () -> s
  put : s -> ()
}

-- 拡張可能なエフェクト（row polymorphism）
effect Reader r | e {
  ask : () ->{e} r
  local : (r -> r) -> (() ->{e} a) -> a
}
```

### 2. エフェクトの拡張

```x
-- 既存のエフェクトを拡張
effect TransactionalState s extends State s {
  checkpoint : () -> ()
  rollback : () -> ()
}

-- 複数のエフェクトを組み合わせ
effect MonadIO extends IO, Error {
  tryIO : (() ->{IO} a) -> Either Text a
}
```

### 3. Open Handlers（拡張可能なハンドラ）

```x
-- 部分的なハンドラ定義
handler stateCore s {
  get() -> resume(s, s)
  put(s') -> resume((), s')
}

-- ハンドラの拡張
handler transactionalState s extends stateCore s {
  -- 継承した操作に加えて
  checkpoint() -> {
    saved := s
    resume(())
  }
  rollback() -> {
    s := saved
    resume(())
  }
}
```

### 4. エフェクトの合成

```x
-- Row polymorphismを使った合成
genericLogger : (() ->{Logger | e} a) ->{e} a
genericLogger action = with logger handle action()

-- 任意のエフェクトと組み合わせ可能
result = genericLogger {
  with state(0) {
    log("Current state: " ++ show(get()))
    put(42)
  }
}
```

## 高度な機能

### 1. Effect Aliasing

```x
-- エフェクトの別名
effect alias Console = {Print, Read}
effect alias Storage = {FileIO, Database}

-- 使用
processData : () ->{Console, Storage} ()
processData = {
  line <- readLine()
  saveToFile("data.txt", line)
}
```

### 2. First-class Effects

```x
-- エフェクトを値として扱う
type EffectHandler e a = {
  handle : (() ->{e} a) -> a
}

-- ハンドラのリスト
handlers : List (EffectHandler e a)
handlers = [
  { handle = with state(0) },
  { handle = with error },
  { handle = with async }
]
```

### 3. Effect Constraints

```x
-- エフェクト制約
processWithConstraints : {e: State Int | Error <: e} => 
                         () ->{e} Int
processWithConstraints = {
  x <- get()
  if x < 0 
    then throw("Invalid state")
    else return x
}
```

### 4. Dynamic Effect Handling

```x
-- 実行時にハンドラを選択
dynamicHandle : Text -> (() ->{IO} a) -> a
dynamicHandle strategy action = 
  match strategy {
    "mock" -> with mockIO handle action()
    "real" -> with realIO handle action()
    "test" -> with testIO handle action()
  }
```

## セマンティックAST表現

```x
-- With expression (semantic AST)
With {
  handlers: [
    Handler {
      kind: Named("state", [Literal(Int(0))]),
      operations: [
        Operation {
          name: "get",
          params: [],
          body: Resume(Var("state"))
        },
        Operation {
          name: "put", 
          params: [Var("s")],
          body: Sequence([
            Assign("state", Var("s")),
            Resume(Unit)
          ])
        }
      ]
    }
  ],
  body: Do([
    Bind("x", Perform("State", "get", [])),
    Expr(Perform("State", "put", [App("+", [Var("x"), Literal(Int(10))])])),
    Expr(Perform("State", "get", []))
  ])
}
```

## 実装例

### 1. 合成可能なロギング

```x
-- 基本ロガー
effect Logger {
  log : Text -> ()
}

-- 拡張ロガー
effect DetailedLogger extends Logger {
  debug : Text -> ()
  error : Text -> ()
}

-- 使用例
app : () ->{DetailedLogger | e} ()
app = {
  log("Starting app")
  debug("Initializing...")
  -- 他のエフェクトも使用可能
}
```

### 2. モジュラーエフェクト

```x
-- モジュールごとにエフェクトを定義
module Database.Effects {
  effect DB {
    query : Text -> List Row
    execute : Text -> ()
  }
}

module Cache.Effects {
  effect Cache {
    get : Text -> Maybe a
    set : Text -> a -> ()
  }
}

-- 組み合わせて使用
import Database.Effects (DB)
import Cache.Effects (Cache)

cachedQuery : Text ->{DB, Cache} List Row
cachedQuery sql = {
  cached <- Cache.get(sql)
  match cached {
    Some(result) -> return result
    None -> {
      result <- DB.query(sql)
      Cache.set(sql, result)
      return result
    }
  }
}
```

### 3. エフェクトのテスト

```x
-- テスト用のモックハンドラ
handler mockState initial {
  -- 状態の変更を記録
  changes : Ref (List (Text, s)) = ref []
  
  get() -> {
    changes := ("get", initial) :: !changes
    resume(initial, initial)
  }
  
  put(s) -> {
    changes := ("put", s) :: !changes
    resume((), s)
  }
  
  return x -> (x, !changes)
}

-- テスト
test "state operations" = {
  (result, log) = with mockState(0) {
    x <- get()
    put(x + 1)
    get()
  }
  
  assert(result == 1)
  assert(log == [("get", 0), ("put", 1), ("get", 1)])
}
```

## 設計の利点

1. **合成可能性**: エフェクトを自由に組み合わせ可能
2. **拡張性**: 既存のエフェクトを拡張して新しい機能を追加
3. **型安全**: すべてのエフェクトが型システムで追跡される
4. **モジュラー**: エフェクトをモジュール単位で管理
5. **テスタブル**: モックハンドラで簡単にテスト可能

この設計により、大規模なアプリケーションでも管理しやすいエフェクトシステムが実現できます。