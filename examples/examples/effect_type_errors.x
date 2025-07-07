# エフェクト型チェックのエラー例と正しい使い方

# ============================================
# 1. ハンドルされていないエフェクト
# ============================================

# ❌ エラー: IO エフェクトがハンドルされていない
badExample1 : () -> Text
badExample1 = readFile "test.txt"
# Error: Unhandled effect 'IO'
# The function performs IO but the type says it's pure

# ✅ 正しい: 型にエフェクトを明記
goodExample1 : () ->{IO} Text
goodExample1 = readFile "test.txt"

# ✅ または: with でハンドル
goodExample1b : () -> Text
goodExample1b = with io handle {
  readFile "test.txt"
}

# ============================================
# 2. エフェクトの伝播
# ============================================

# ❌ エラー: 呼び出し先のエフェクトを無視
badHelper : Text -> Int
badHelper path = 
  length (readFile path)  # readFile は IO エフェクトを持つ
# Error: Unhandled effect 'IO' in function body

# ✅ 正しい: エフェクトを型に含める
goodHelper : Text ->{IO} Int
goodHelper path = 
  content <- readFile path
  return (length content)

# ============================================
# 3. 複数エフェクトの処理
# ============================================

# 関数が複数のエフェクトを使用
processData : Text ->{IO, Error} Int
processData path = do
  content <- readFile path    # IO
  number <- parseInt content  # Error
  return number

# ❌ エラー: 一部のエフェクトしかハンドルしていない
badUsage : () -> Int
badUsage = with error handle {
  processData "data.txt"  # IO がハンドルされていない
}
# Error: Unhandled effect 'IO'

# ✅ 正しい: すべてのエフェクトをハンドル
goodUsage : () -> Int
goodUsage = with io with error handle {
  processData "data.txt"
}

# ✅ または: 一部を伝播
partialHandle : () ->{IO} Either Text Int
partialHandle = with error handle {
  processData "data.txt"  # Error はハンドル、IO は伝播
}

# ============================================
# 4. エフェクト制約
# ============================================

# エフェクト制約を持つ関数
needsState : {e: State Int | e} => () ->{e} Int
needsState = do
  x <- get()
  put(x * 2)
  get()

# ❌ エラー: 必要なエフェクトが利用できない
badConstraint : () ->{Logger} Int
badConstraint = needsState()
# Error: Effect constraint not satisfied
# Required: State Int
# Available: Logger

# ✅ 正しい: 必要なエフェクトを提供
goodConstraint : () ->{State Int, Logger} Int
goodConstraint = needsState()

# ============================================
# 5. ハンドラの型不一致
# ============================================

# State エフェクトは Int を期待
increment : () ->{State Int} Int
increment = do
  x <- get()
  put(x + 1)
  get()

# ❌ エラー: 間違った型でハンドラを初期化
badHandler : () -> Int
badHandler = with state("not a number") handle {
  increment()
}
# Error: Handler type mismatch
# Expected: Int
# Found: Text

# ✅ 正しい: 正しい型で初期化
goodHandler : () -> Int
goodHandler = with state(0) handle {
  increment()
}

# ============================================
# 6. Row Polymorphism のエラー
# ============================================

# 2つの異なる row variable
func1 : {e1} => () ->{Logger | e1} ()
func2 : {e2} => () ->{Error | e2} ()

# ❌ エラー: row variable が統一できない
badCompose : {e} => () ->{e} ()
badCompose = do
  func1()  # requires {Logger | e1}
  func2()  # requires {Error | e2}
# Error: Cannot unify row variables e1 and e2

# ✅ 正しい: 両方のエフェクトを含む型
goodCompose : {e} => () ->{Logger, Error | e} ()
goodCompose = do
  func1()
  func2()

# ============================================
# 7. エフェクトの順序
# ============================================

# ハンドラの順序が重要な場合
stateAndError : () -> Either Text Int
stateAndError = with state(0) with error handle {
  x <- get()
  if x < 0 then throw "Negative"
  else return x
}

# 順序を変えると型が変わる
errorAndState : () -> (Either Text Int, Int)
errorAndState = with error with state(0) handle {
  x <- get()
  if x < 0 then throw "Negative"
  else return x
}
# with error が外側なので、状態も結果に含まれる

# ============================================
# 8. 再帰関数のエフェクト
# ============================================

# ❌ エラー: 再帰呼び出しのエフェクトを忘れる
badRecursive : List Text -> List Int
badRecursive files = 
  match files {
    [] -> []
    f :: fs -> 
      content <- readFile f  # IO エフェクト！
      parseInt content :: badRecursive fs
  }
# Error: Unhandled effect 'IO'

# ✅ 正しい: エフェクトを型に含める
goodRecursive : List Text ->{IO, Error} List Int
goodRecursive files = 
  match files {
    [] -> []
    f :: fs -> do
      content <- readFile f
      n <- parseInt content
      rest <- goodRecursive fs
      return (n :: rest)
  }

# ============================================
# 9. 高階関数とエフェクト
# ============================================

# ❌ エラー: 関数引数のエフェクトを考慮していない
badMap : (a -> b) -> List a -> List b
badMap f list = 
  match list {
    [] -> []
    x :: xs -> f x :: badMap f xs
  }

# 使用時にエラー
usesBadMap = badMap readFile ["a.txt", "b.txt"]
# Error: Function 'readFile' has effect IO but badMap expects pure function

# ✅ 正しい: エフェクトを持つ関数も受け取れるように
goodMap : {e} => (a ->{e} b) -> List a ->{e} List b
goodMap f list = 
  match list {
    [] -> []
    x :: xs -> do
      y <- f x
      ys <- goodMap f xs
      return (y :: ys)
  }

# ============================================
# 10. エフェクトのエスケープ
# ============================================

# ❌ エラー: ハンドラの外でエフェクトを使おうとする
badEscape : () -> (() -> Int)
badEscape = 
  with state(0) handle {
    return (\() -> get())  # get はハンドラの外で使えない
  }
# Error: Effect 'State' escapes its handler scope

# ✅ 正しい: 値を計算してから返す
goodNoEscape : () -> Int
goodNoEscape = 
  with state(0) handle {
    x <- get()
    return x
  }