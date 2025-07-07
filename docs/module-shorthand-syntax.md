# モジュール簡潔記法

## 概要

x言語では、モジュール定義を従来のブロック形式に加えて、ドット記法を使った簡潔な形式でも記述できます。内部的にはハッシュで管理され、効率的なアクセスが可能です。

## 基本構文

### 従来のブロック形式

```x
module App {
  export { get, post, put, delete }
  
  get : Request -> Response
  get req = {
    # 実装
  }
  
  post : Request -> Response
  post req = {
    # 実装
  }
}
```

### 新しい簡潔記法

```x
App.get : Request -> Response
App.get req = {
  # 実装
}

App.post : Request -> Response
App.post req = {
  # 実装
}
```

## 詳細な仕様

### 1. 基本的な定義

```x
# 値の定義
Math.pi : Float
Math.pi = 3.14159

# 関数の定義
Math.add : Int -> Int -> Int
Math.add x y = x + y

# 型定義
Math.Complex : Type
Math.Complex = { real: Float, imag: Float }
```

### 2. ネストしたモジュール

```x
# ネストしたモジュールもドット記法で表現
Core.List.map : (a -> b) -> List a -> List b
Core.List.map f list = {
  match list {
    Nil -> Nil
    Cons x xs -> Cons (f x) (Core.List.map f xs)
  }
}

Core.List.filter : (a -> Bool) -> List a -> List a
Core.List.filter pred list = {
  match list {
    Nil -> Nil
    Cons x xs -> {
      if pred x {
        Cons x (Core.List.filter pred xs)
      } else {
        Core.List.filter pred xs
      }
    }
  }
}
```

### 3. エクスポート制御

```x
# publicな関数（デフォルト）
String.length : String -> Int
String.length s = # 実装

# privateな関数（モジュール内部でのみ使用）
private String.validate : String -> Bool
private String.validate s = # 実装

# 明示的なpublic指定も可能
pub String.toUpper : String -> String
pub String.toUpper s = # 実装
```

### 4. 型とエフェクトの定義

```x
# 型定義
Http.Request = {
  method : Method
  path : String
  headers : Map String String
  body : Maybe Bytes
}

Http.Response = {
  status : Int
  headers : Map String String
  body : Bytes
}

# エフェクト定義
Http.effect IO {
  send : Request -> Response
  receive : () -> Request
}
```

### 5. パターンマッチングとの組み合わせ

```x
# パターンマッチングを使った定義
List.head : List a -> Maybe a
List.head Nil = None
List.head (Cons x _) = Some x

List.tail : List a -> Maybe (List a)
List.tail Nil = None
List.tail (Cons _ xs) = Some xs
```

## 利点

### 1. 簡潔性

従来の方法：
```x
module StringUtils {
  export { trim, split, join }
  
  trim : String -> String
  trim s = # 実装
  
  split : Char -> String -> List String
  split delimiter s = # 実装
  
  join : String -> List String -> String
  join separator strings = # 実装
}
```

新しい方法：
```x
StringUtils.trim : String -> String
StringUtils.trim s = # 実装

StringUtils.split : Char -> String -> List String
StringUtils.split delimiter s = # 実装

StringUtils.join : String -> List String -> String
StringUtils.join separator strings = # 実装
```

### 2. 段階的な定義

モジュールの内容を段階的に定義できます：

```x
# ファイル1: math_basic.x
Math.add : Int -> Int -> Int
Math.add x y = x + y

Math.sub : Int -> Int -> Int
Math.sub x y = x - y

# ファイル2: math_advanced.x
Math.sqrt : Float -> Float
Math.sqrt x = # 実装

Math.pow : Float -> Float -> Float
Math.pow base exp = # 実装
```

### 3. 選択的インポート

```x
# 特定の関数のみインポート
import Math.sqrt
import Math.pow

# 使用
let x = sqrt 16
let y = pow 2 8
```

## 内部実装

### ハッシュベースの管理

```x
# 内部的には以下のような構造で管理
ModuleTable = HashMap<ModuleHash, ModuleEntry>

ModuleHash = hash(module_path + member_name)

ModuleEntry = {
  path : ModulePath      # e.g., ["Math"]
  member : String        # e.g., "sqrt"
  value : Value          # 実際の値や関数
  visibility : Visibility
  metadata : Metadata
}
```

### 名前解決

```x
# Math.sqrt を解決する場合
1. "Math.sqrt" を ModuleHash に変換
2. ModuleTable から対応する ModuleEntry を検索
3. visibility をチェック
4. value を返す
```

## 移行ガイド

### 既存コードからの移行

```x
# Before
module Utils {
  export { helper1, helper2 }
  
  let helper1 = # 実装
  let helper2 = # 実装
}

# After
Utils.helper1 = # 実装
Utils.helper2 = # 実装
```

### 混在使用

両方の記法を同じプロジェクトで使用可能：

```x
# ブロック形式（複雑なモジュール向け）
module ComplexModule {
  # privateな状態
  let state = ref 0
  
  export { increment, decrement, getValue }
  
  increment : () -> ()
  increment = # 実装
  
  decrement : () -> ()
  decrement = # 実装
  
  getValue : () -> Int
  getValue = # 実装
}

# ドット記法（シンプルなユーティリティ向け）
SimpleUtils.double : Int -> Int
SimpleUtils.double x = x * 2

SimpleUtils.triple : Int -> Int
SimpleUtils.triple x = x * 3
```

## ベストプラクティス

1. **シンプルなユーティリティ**: ドット記法を使用
2. **状態を持つモジュール**: ブロック形式を使用
3. **大規模なAPI**: ブロック形式で構造化
4. **単一責任の関数群**: ドット記法で簡潔に

## 例: HTTPサーバー

```x
# ルーティング
Server.route : Method -> String -> Handler -> ()
Server.route method path handler = # 実装

# ハンドラー定義
Server.get : String -> Handler -> ()
Server.get = Server.route GET

Server.post : String -> Handler -> ()
Server.post = Server.route POST

# 使用例
Server.get "/users" \req -> {
  json (getAllUsers())
}

Server.post "/users" \req -> {
  user <- parseJson req.body
  id <- createUser user
  json { id = id }
}

# サーバー起動
Server.listen : Int -> () -> {IO} ()
Server.listen port callback = # 実装

# main
main = Server.listen 8080 \() -> {
  print "Server started on port 8080"
}
```

この記法により、x言語はより柔軟で表現力豊かなモジュールシステムを提供します。