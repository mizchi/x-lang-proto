# 電卓アプリケーション（エフェクトシステムを活用）

module Calculator {
  export { calculate, evaluateExpression, calculatorREPL }
  
  # ============================================
  # 式の定義
  # ============================================
  
  type Expr = {
    Num(Float)
    Add(Expr, Expr)
    Sub(Expr, Expr)
    Mul(Expr, Expr)
    Div(Expr, Expr)
    Pow(Expr, Expr)
    Var(Text)
    Let(Text, Expr, Expr)
  }
  
  # ============================================
  # エフェクト定義
  # ============================================
  
  effect Calculator {
    getVar : Text -> Float
    setVar : Text -> Float -> ()
  }
  
  effect MathError {
    divByZero : () -> a
    undefined : Text -> a
    overflow : () -> a
  }
  
  # ============================================
  # 評価関数
  # ============================================
  
  evaluate : Expr ->{Calculator, MathError} Float
  evaluate expr = {
    match expr {
      Num(n) -> { n }
      
      Add(left, right) -> {
        let l = { evaluate(left) }
        let r = { evaluate(right) }
        checkOverflow(l + r)
      }
      
      Sub(left, right) -> {
        let l = { evaluate(left) }
        let r = { evaluate(right) }
        l - r
      }
      
      Mul(left, right) -> {
        let l = { evaluate(left) }
        let r = { evaluate(right) }
        checkOverflow(l * r)
      }
      
      Div(left, right) -> {
        let l = { evaluate(left) }
        let r = { evaluate(right) }
        if r == 0.0 {
          divByZero()
        } else {
          l / r
        }
      }
      
      Pow(base, exp) -> {
        let b = { evaluate(base) }
        let e = { evaluate(exp) }
        checkOverflow(pow(b, e))
      }
      
      Var(name) -> {
        getVar(name)
      }
      
      Let(name, value, body) -> {
        let v = { evaluate(value) }
        setVar(name, v)
        evaluate(body)
      }
    }
  }
  
  # オーバーフローチェック
  checkOverflow : Float ->{MathError} Float
  checkOverflow n = {
    if n > 1e308 || n < -1e308 {
      overflow()
    } else {
      n
    }
  }
  
  # ============================================
  # ハンドラ
  # ============================================
  
  # 変数環境を管理するハンドラ
  handler variableHandler env {
    getVar(name) -> {
      match lookup(name, env) {
        Some(value) -> { resume(value) }
        None -> { undefined(name) }
      }
    }
    
    setVar(name, value) -> {
      let newEnv = { insert(name, value, env) }
      with variableHandler(newEnv) { resume(()) }
    }
    
    return x -> { x }
  }
  
  # エラーハンドラ
  handler errorHandler {
    divByZero() -> { Left("Error: Division by zero") }
    undefined(name) -> { Left("Error: Undefined variable '" ++ name ++ "'") }
    overflow() -> { Left("Error: Number overflow") }
    return x -> { Right(x) }
  }
  
  # ============================================
  # パーサー
  # ============================================
  
  # トークン型
  type Token = {
    TNum(Float)
    TPlus
    TMinus
    TMul
    TDiv
    TPow
    TLParen
    TRParen
    TVar(Text)
    TLet
    TEquals
    TIn
    TEnd
  }
  
  # 簡易パーサー（エラー処理付き）
  parseExpression : Text ->{Error} Expr
  parseExpression input = {
    let tokens = { tokenize(input) }
    parseExpr(tokens)
  }
  
  # ============================================
  # 計算機能
  # ============================================
  
  # 式を評価（すべてのエフェクトをハンドル）
  calculate : Expr -> Either Text Float
  calculate expr = {
    with errorHandler with variableHandler([]) {
      evaluate(expr)
    }
  }
  
  # テキストから評価
  evaluateExpression : Text -> Either Text Float
  evaluateExpression input = {
    with error {
      expr <- parseExpression(input)
      return calculate(expr)
    }
  }
  
  # ============================================
  # REPL (Read-Eval-Print Loop)
  # ============================================
  
  type ReplState = {
    variables : List (Text, Float)
    history : List (Text, Float)
  }
  
  calculatorREPL : () ->{IO, State ReplState} ()
  calculatorREPL = do {
    print("Calculator v1.0")
    print("Type 'help' for commands, 'quit' to exit\n")
    
    loop(do {
      print("> ")
      input <- readLine()
      
      match input {
        "quit" -> { break() }
        "help" -> { printHelp() }
        "vars" -> { printVariables() }
        "history" -> { printHistory() }
        "clear" -> { clearVariables() }
        _ -> {
          processExpression(input)
        }
      }
    })
  }
  
  # 式を処理
  processExpression : Text ->{IO, State ReplState} ()
  processExpression input = {
    state <- get()
    
    let result = with errorHandler with variableHandler(state.variables) {
      expr <- parseExpression(input)
      value <- evaluate(expr)
      
      # 変数環境を更新
      let newVars = { getCurrentVars() }
      put(state with { 
        variables = newVars
        history = (input, value) :: state.history
      })
      
      return value
    }
    
    match result {
      Right(value) -> { print(show(value)) }
      Left(error) -> { print(error) }
    }
  }
  
  # ============================================
  # ヘルパー関数
  # ============================================
  
  printHelp : () ->{IO} ()
  printHelp = do {
    print("Commands:")
    print("  help     - Show this help")
    print("  vars     - Show all variables")
    print("  history  - Show calculation history")
    print("  clear    - Clear all variables")
    print("  quit     - Exit calculator")
    print("\nExamples:")
    print("  2 + 3 * 4")
    print("  let x = 10 in x * x")
    print("  x ^ 2 + 2 * x + 1")
  }
  
  # ============================================
  # テスト
  # ============================================
  
  test "basic arithmetic" = {
    assert(calculate(Add(Num(2.0), Num(3.0))) == Right(5.0))
    assert(calculate(Mul(Num(4.0), Num(5.0))) == Right(20.0))
    assert(calculate(Sub(Num(10.0), Num(3.0))) == Right(7.0))
  }
  
  test "division by zero" = {
    match calculate(Div(Num(1.0), Num(0.0))) {
      Left(msg) -> { assert(contains("Division by zero", msg)) }
      Right(_) -> { assert(false) }
    }
  }
  
  test "variables" = {
    let expr = { Let("x", Num(10.0), 
                   Let("y", Num(20.0), 
                     Add(Var("x"), Var("y")))) }
    assert(calculate(expr) == Right(30.0))
  }
  
  test "undefined variable" = {
    match calculate(Var("unknown")) {
      Left(msg) -> { assert(contains("Undefined variable", msg)) }
      Right(_) -> { assert(false) }
    }
  }
}