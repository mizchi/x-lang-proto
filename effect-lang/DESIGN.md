# EffectLang - エフェクトシステムを持つ関数型言語

## 言語設計概要

**EffectLang**は、エフェクトシステムを中核とする静的型付け関数型プログラミング言語です。

### 核となる概念

#### 1. **エフェクトシステム**
```effectlang
// 純粋関数（エフェクトなし）
pure add : Int -> Int -> Int
pure add x y = x + y

// IOエフェクトを持つ関数
effect IO
io print : String -> Unit <IO>
io print msg = builtin_print msg

// 複数エフェクト
effect State[s], IO
stateful_io increment_and_log : () -> Int <State[Int], IO>
```

#### 2. **代数的エフェクトとハンドラー**
```effectlang
// エフェクト定義
effect State[s] {
  get : () -> s
  put : s -> ()
}

effect Except[e] {
  throw : e -> !a  // never returns
}

// ハンドラー
handle_state : forall a s. s -> Computation[a, State[s]] -> (a, s)
handle_state initial comp = 
  handle comp {
    return x -> (x, initial)
    get () k -> handle_state initial (k initial)  
    put s k -> handle_state s (k ())
  }
```

#### 3. **高階型とカインド**
```effectlang
// 基本型
Int : Type
String : Type
List : Type -> Type
Effect : Type -> Type

// 高階型
Functor : (Type -> Type) -> Type
Monad : (Type -> Type) -> Type
Effect : (Type -> Type) -> Type
```

#### 4. **型推論 (Hindley-Milner + Effects)**
```effectlang
// 型アノテーションは省略可能
let map f xs = match xs {
  [] -> []
  h :: t -> f h :: map f t
}
// 推論される型: forall a b e. (a -> b <e>) -> List[a] -> List[b] <e>
```

### 構文設計

#### 基本構文
```effectlang
// 値定義
let x = 42
let name = "Alice"

// 関数定義
let add x y = x + y
let factorial n = if n <= 1 then 1 else n * factorial (n - 1)

// 型アノテーション
let id : forall a. a -> a = fun x -> x

// エフェクトアノテーション
let print_twice : String -> Unit <IO> = fun msg ->
  do {
    print msg;
    print msg
  }
```

#### パターンマッチング
```effectlang
// データ型定義
data List[a] = Nil | Cons a (List[a])
data Option[a] = None | Some a
data Result[a, e] = Ok a | Error e

// パターンマッチ
let head = fun xs -> match xs {
  Nil -> None
  Cons h _ -> Some h
}
```

#### エフェクト処理
```effectlang
// do記法
let program = do {
  x <- get;
  put (x + 1);
  y <- get;
  print (show y);
  return y
}

// ハンドラー
let main = 
  program 
  |> handle_state 0
  |> handle_io
```

### エフェクトシステム詳細

#### エフェクト行 (Effect Rows)
```effectlang
// 開いたエフェクト行
f : a -> b <e | r>  // eエフェクト + 他のエフェクトr

// 閉じたエフェクト行  
g : a -> b <IO, State[Int]>  // IOとState[Int]のみ

// エフェクト多相
pure_map : forall a b e. (a -> b <e>) -> List[a] -> List[b] <e>
```

#### エフェクトアライアス
```effectlang
effect Console = IO, State[String]
effect WebApp = HTTP, DB, Console

web_handler : forall a. Computation[a, WebApp] -> IO (Result[a, Error])
```

#### ライフタイム (リソース管理)
```effectlang
effect Resource[r] {
  acquire : () -> r
  release : r -> ()
}

with_file : forall a. String -> (File -> a <e>) -> a <Resource[File], e>
```

### 型システム

#### 基本型
```
τ ::= α                    // 型変数
    | c                    // 型定数 (Int, String, ...)
    | τ₁ -> τ₂ <ε>         // 関数型 (エフェクト付き)
    | τ₁ τ₂                // 型適用
    | ∀α.τ                 // 全称量化
    | μα.τ                 // 再帰型
```

#### エフェクト型
```
ε ::= ∅                    // 空エフェクト (純粋)
    | {eff₁, ..., effₙ}   // エフェクト集合
    | α                    // エフェクト変数
    | ε₁ ∪ ε₂              // エフェクト合成
```

#### カインド
```
κ ::= Type                 // 値型のカインド
    | κ₁ -> κ₂             // 型コンストラクターのカインド
    | Effect               // エフェクトのカインド
    | Row                  // エフェクト行のカインド
```

### コンパイル戦略

#### 1. **エフェクト消去**
```effectlang
// ソース
f : Int -> Int <State[String]>
f x = do { s <- get; put (s ++ show x); return (x + 1) }

// コンパイル後 (継続渡し)
f_compiled : Int -> String -> Cont[Int, String]
f_compiled x s = \k -> k (x + 1) (s ++ show x)
```

#### 2. **最適化**
- **エフェクト特殊化**: 使用されるエフェクトのみを残す
- **インライン化**: 小さなハンドラーのインライン展開
- **デッドコード除去**: 不要なエフェクト処理の削除

### 標準ライブラリ設計

```effectlang
// Core effects
effect IO {
  read_line : () -> String
  print : String -> ()
  read_file : String -> String  
  write_file : String -> String -> ()
}

effect State[s] {
  get : () -> s
  put : s -> ()
  modify : (s -> s) -> ()
}

effect Except[e] {
  throw : e -> !a
  catch : Computation[a, Except[e]] -> (e -> a) -> a
}

// Async/concurrent effects
effect Async {
  fork : Computation[(), e] -> ThreadId
  yield : () -> ()
  sleep : Duration -> ()
}

effect Channel[a] {
  send : a -> ()
  receive : () -> a
}
```

### 実装戦略 (Rust)

#### アーキテクチャ
```
effect-lang/
├── syntax/          # 構文定義・パーサー
├── types/           # 型システム・推論
├── effects/         # エフェクトシステム
├── compile/         # コンパイラー
├── runtime/         # ランタイムシステム
└── std/            # 標準ライブラリ
```

#### 主要コンポーネント
1. **Parser**: S式ベース + 高レベル構文サポート
2. **Type Checker**: Hindley-Milner + Effect rows
3. **Effect System**: 代数的エフェクト + ハンドラー
4. **Code Generator**: 継続渡しスタイル変換
5. **Runtime**: エフェクトランタイム + GC

### 言語の特徴

#### 優位性
✅ **エフェクト安全性**: 副作用の型レベル追跡  
✅ **合成可能性**: エフェクトハンドラーの組み合わせ  
✅ **パフォーマンス**: ゼロコスト抽象化（コンパイル時消去）  
✅ **表現力**: 代数的エフェクト > モナド  
✅ **推論**: 型とエフェクトの自動推論  

#### 応用分野
- **並行プログラミング**: Async/awaitを型安全に
- **状態管理**: 複雑な状態の型安全な管理
- **エラーハンドリング**: 例外の型レベル追跡
- **リソース管理**: ファイル・メモリの安全な管理
- **DSL構築**: ドメイン特化言語の基盤

### 実装フェーズ

#### Phase 1: Core Language
- [x] 基本構文・AST
- [ ] 型システム (Hindley-Milner)
- [ ] 基本的なエフェクトシステム
- [ ] インタープリター

#### Phase 2: Advanced Features  
- [ ] 代数的エフェクト・ハンドラー
- [ ] エフェクト行・多相性
- [ ] 最適化エンジン
- [ ] コード生成

#### Phase 3: Production
- [ ] 標準ライブラリ
- [ ] ツールチェーン
- [ ] エコシステム

この設計により、**学術的に興味深く、実用的でもある**エフェクトシステム言語を作成します。