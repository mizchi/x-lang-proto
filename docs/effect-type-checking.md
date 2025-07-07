# エフェクト型チェックの設計

## 概要

x言語のエフェクト型システムは、すべての効果的な操作が適切なハンドラによって処理されることを保証します。

## 基本原則

### 1. エフェクトの追跡

```x
-- 関数の型にエフェクトが含まれる
readFile : Text ->{IO} Text
parseInt : Text ->{Error} Int
getState : () ->{State s} s

-- 複数のエフェクト
processFile : Text ->{IO, Error} Int
processFile path = do
  content <- readFile path  -- IO エフェクト
  parseInt content          -- Error エフェクト
```

### 2. ハンドラの要求

```x
-- これはエラー: IO エフェクトがハンドルされていない
main : () -> ()
main = readFile "test.txt"  -- ❌ Unhandled effect: IO

-- 正しい: with でハンドル
main : () -> ()
main = with io handle {
  content <- readFile "test.txt"  -- ✅ IO is handled
  print content
}
```

## 型チェックルール

### 1. エフェクトの伝播

```x
-- エフェクトは呼び出し元に伝播する
helper : Text ->{IO} Int
helper path = do
  content <- readFile path  -- IO エフェクト
  return (length content)

-- caller も IO エフェクトを持つ
caller : () ->{IO} Int
caller = helper "data.txt"
```

### 2. ハンドラによる除去

```x
-- with ハンドラがエフェクトを除去
pureResult : Int
pureResult = with state(0) handle {
  x <- get()         -- State エフェクト
  put(x + 1)         -- State エフェクト
  get()              -- State エフェクト
}  -- ここで State エフェクトが除去される
```

### 3. 部分的なハンドリング

```x
-- 一部のエフェクトだけハンドル
partialHandle : () ->{IO} Either Text Int
partialHandle = with error handle {
  content <- readFile "data.txt"  -- IO は残る
  n <- parseInt content            -- Error はハンドルされる
  return n
}
```

## Row Polymorphism

### 1. エフェクトの合成

```x
-- 任意のエフェクトと組み合わせ可能
logging : {e} => (() ->{Logger | e} a) ->{e} a
logging action = with logger handle action()

-- 使用例
withLogging : () ->{IO} ()
withLogging = logging {
  content <- readFile "test.txt"  -- IO エフェクトは保持される
  log("Read file")                -- Logger エフェクトは除去される
  print content
}
```

### 2. エフェクト制約

```x
-- エフェクトに制約を設定
needsState : {e: State Int | e} => () ->{e} Int
needsState = do
  x <- get()
  put(x + 1)
  get()

-- OK: State Int を含む
valid : () ->{State Int, Logger} Int
valid = needsState()

-- エラー: State Int を含まない
invalid : () ->{Logger} Int
invalid = needsState()  -- ❌ Effect constraint not satisfied
```

## 型チェックアルゴリズム

### 1. エフェクト収集

```rust
// エフェクトを収集しながら型チェック
fn check_expr(expr: &Expr, ctx: &mut Context) -> Result<EffectType> {
    match expr {
        Expr::Perform { effect, op, args } => {
            // エフェクトを現在のコンテキストに追加
            ctx.add_effect(effect);
            // 操作の型をチェック
            check_operation(effect, op, args)
        }
        Expr::With { handlers, body } => {
            // ハンドラスコープに入る
            ctx.enter_handlers(handlers);
            let body_type = check_expr(body, ctx)?;
            ctx.exit_handlers();
            // ハンドルされたエフェクトを除去
            Ok(remove_handled_effects(body_type, handlers))
        }
        // ...
    }
}
```

### 2. 関数適用のチェック

```rust
fn check_application(
    func: &EffectType,
    args: &[EffectType],
    ctx: &Context
) -> Result<EffectType> {
    // 関数が要求するエフェクトが利用可能か確認
    let required_effects = func.effects;
    let available_effects = ctx.get_available_effects();
    
    if !required_effects.is_subset_of(available_effects) {
        return Err(UnhandledEffects {
            required,
            available
        });
    }
    
    // 引数のエフェクトも収集
    let all_effects = union_effects([func.effects, args.effects]);
    Ok(EffectType {
        typ: func.return_type,
        effects: all_effects
    })
}
```

## エラーメッセージ

### 1. ハンドルされていないエフェクト

```
Error: Unhandled effect 'IO'
  |
5 |   content <- readFile "test.txt"
  |              ^^^^^^^^^^^^^^^^^^^
  |
  = This expression performs the 'IO' effect
  = Available handlers: [State, Error]
  = Suggestion: Add 'with io handle { ... }' around this expression
```

### 2. エフェクト制約違反

```
Error: Effect constraint not satisfied
   |
10 |   result = needsState()
   |            ^^^^^^^^^^^^
   |
   = Function requires: State Int
   = Available effects: [Logger]
   = Suggestion: Add State handler or change function signature
```

### 3. ハンドラの不一致

```
Error: Handler type mismatch
  |
7 | with state("not a number") handle {
  |            ^^^^^^^^^^^^^^
  |
  = Handler 'state' expects type: Int
  = Found type: Text
```

## 実装の詳細

### EffectRow 型

```rust
struct EffectRow {
    effects: HashSet<Effect>,
    row_var: Option<TypeVar>,  // for polymorphism
}
```

### エフェクトコンテキスト

```rust
struct EffectContext {
    handlers: Vec<HandlerScope>,
    current_effects: EffectRow,
}
```

### 型推論との統合

```x
-- 型とエフェクトを同時に推論
inferred = do
  x <- get()      -- x: α, effect: State α
  put(x + 1)      -- α = Int, effect: State Int
  get()           -- result: Int, effect: State Int
-- 推論結果: () ->{State Int} Int
```

## 利点

1. **コンパイル時保証**: すべてのエフェクトが処理される
2. **明示的**: 関数のエフェクトが型に現れる
3. **合成可能**: エフェクトを自由に組み合わせ可能
4. **局所的**: ハンドラのスコープが明確
5. **多相的**: Row polymorphismで柔軟な抽象化

この型システムにより、副作用を持つプログラムも安全に記述できます。