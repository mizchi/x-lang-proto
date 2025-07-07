# x Language: ブレースファーストスタイルの例

# ============================================
# 基本的な値と関数
# ============================================

# 値定義
pi : Float;
pi = { 3.14159 };

# 関数定義
square : Int -> Int;
square x = { x * x };

# 複数引数の関数
add : Int -> Int -> Int;
add x y = { x + y };

# ============================================
# 制御構造
# ============================================

# if式
abs : Int -> Int;
abs x = {
  if x < 0 {
    -x
  } else {
    x
  }
};

# ネストしたif
sign : Int -> Int;
sign x = {
  if x < 0 {
    -1
  } else {
    if x > 0 {
      1
    } else {
      0
    }
  }
};

# ============================================
# パターンマッチ
# ============================================

type Maybe a = {
  None;
  Some(a)
};

# match式
unwrapOr : Maybe a -> a -> a;
unwrapOr opt default = {
  match opt {
    None -> { default };
    Some(x) -> { x }
  }
};

# リストのパターンマッチ
type List a = {
  Nil;
  Cons(a, List a)
};

length : List a -> Int;
length list = {
  match list {
    Nil -> { 0 };
    Cons(_, xs) -> { 1 + length(xs) }
  }
};

# ============================================
# let式
# ============================================

distance : Int -> Int -> Float;
distance x y = {
  let {
    dx = x - 0;
    dy = y - 0;
    squared = dx * dx + dy * dy
  };
  sqrt(Float(squared))
};

# ============================================
# do記法
# ============================================

readAndParse : Text ->{IO, Error} Int
readAndParse filename = do {
  content <- readFile(filename)
  lines <- split('\n', content)
  
  match lines {
    [] -> { throw("Empty file") }
    first :: _ -> { parseInt(first) }
  }
}

# ============================================
# with構文（エフェクトハンドラ）
# ============================================

# 単一ハンドラ
simpleState : Int;
simpleState = with state(42) {
  x <- get();
  put(x * 2);
  get()
};

# 複数ハンドラ
complexComputation : Either Text Int;
complexComputation = with error with state(0) {
  put(10);
  x <- get();
  
  if x > 5 {
    put(x * 2);
    get()
  } else {
    throw("Value too small")
  }
};

# ============================================
# パイプライン演算子
# ============================================

processData : List Int -> Int;
processData data = {
  data
    |> filter(x -> { x > 0 })
    |> map(x -> { x * 2 })
    |> fold(0, (acc, x) -> { acc + x })
};

# ============================================
# 高階関数
# ============================================

compose : (b -> c) -> (a -> b) -> (a -> c);
compose f g = {
  x -> { f(g(x)) }
};

# カリー化された関数
curry : ((a, b) -> c) -> a -> b -> c;
curry f = {
  x -> { y -> { f(x, y) } }
};

# ============================================
# エフェクト定義
# ============================================

effect State s {
  get : () -> s;
  put : s -> ()
};

effect Error {
  throw : Text -> a;
  catch : (() -> a) -> (Text -> a) -> a
};

effect Async {
  await : Promise a -> a;
  async : (() -> a) -> Promise a
};

# ============================================
# ハンドラ定義
# ============================================

handler stateHandler s {
  get() -> { resume(s, s) };
  put(s') -> { resume((), s') };
  return x -> { (x, s) }
};

handler errorHandler {
  throw(msg) -> { Left(msg) };
  return x -> { Right(x) }
};

# ============================================
# 型エイリアスと代数的データ型
# ============================================

type Result e a = Either e a;

type Tree a = {
  Leaf(a);
  Branch(Tree a, Tree a)
};

mapTree : (a -> b) -> Tree a -> Tree b;
mapTree f tree = {
  match tree {
    Leaf(x) -> { Leaf(f(x)) };
    Branch(l, r) -> {
      Branch(mapTree(f, l), mapTree(f, r))
    }
  }
};

# ============================================
# モジュール定義
# ============================================

module Utils {
  export { map, filter, fold };
  
  map : (a -> b) -> List a -> List b;
  map f list = {
    match list {
      Nil -> { Nil };
      Cons(x, xs) -> { Cons(f(x), map(f, xs)) }
    }
  };
  
  filter : (a -> Bool) -> List a -> List a;
  filter pred list = {
    match list {
      Nil -> { Nil };
      Cons(x, xs) -> {
        if pred(x) {
          Cons(x, filter(pred, xs))
        } else {
          filter(pred, xs)
        }
      }
    }
  };
  
  fold : (b -> a -> b) -> b -> List a -> b;
  fold f init list = {
    match list {
      Nil -> { init };
      Cons(x, xs) -> { fold(f, f(init, x), xs) }
    }
  }
}

# ============================================
# 実用的な例：Webサーバー
# ============================================

type Request = {
  method : Text;
  path : Text;
  headers : List (Text, Text);
  body : Text
};

type Response = {
  status : Int;
  headers : List (Text, Text);
  body : Text
};

handleRequest : Request ->{IO, Error, Async} Response;
handleRequest req = {
  match (req.method, req.path) {
    ("GET", "/") -> {
      Response(200, [("Content-Type", "text/html")], "<h1>Hello!</h1>")
    };
    ("GET", "/api/data") -> do {
      data <- fetchFromDatabase();
      json <- toJson(data);
      Response(200, [("Content-Type", "application/json")], json)
    };
    ("POST", "/api/users") -> do {
      user <- parseJson(req.body);
      id <- saveUser(user);
      Response(201, [], "User created: " ++ show(id))
    };
    _ -> {
      Response(404, [], "Not found")
    }
  }
};