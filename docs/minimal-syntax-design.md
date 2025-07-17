# x言語 最小構文設計案

## 設計思想

1. **木構造を第一に考える** - テキスト表現は二次的
2. **最小限のパース規則** - 3種類のノードのみ
3. **型は推論後にインライン化** - 人間が書く必要なし
4. **関数ごとに独立したハッシュ** - コンテンツアドレス指向

## AST定義（3種類のみ）

```rust
enum Expr {
    Atom(String),              // 数値、文字列、識別子
    List(Vec<Expr>),          // リスト/適用
    Ann(Box<Expr>, Type),     // 型注釈（推論後に自動付与）
}
```

## 構文規則

### 基本規則

1. **アトム**: 数値、文字列、識別子
2. **リスト**: 括弧で囲む
3. **適用**: 最初の要素が関数、残りが引数

### S式構文

```x
; 完全なS式
(add (mul 2 3) 4)
```

### 関数定義

```x
; 完全なS式
(def factorial n
  (if (eq n 0)
    1
    (mul n (factorial (sub n 1)))))

```

### 型注釈（AI/推論後に自動付与）

```x
; 推論前
(def add x y
  (+ x y))

; 推論後（型がインライン化される）
(def add x y : (Int Int -> Int)
  (+ x y : Int))
```

## 特殊フォーム（最小限）

```x
def     ; 定義
let     ; 局所束縛
if      ; 条件分岐
match   ; パターンマッチ
lambda  ; 無名関数（または \ ）
```

## モジュールとバージョン

```x
; モジュール定義
module Math v1.0.0

; 関数は自動的にハッシュが付与される
def add x y          ; => add#a3f5c2b1
  + x y

; インポート時にバージョン指定
import Math@"^1.0.0" add
```

## 内部表現

### 1. パース段階
```x
add mul 2 3 4
```
↓
```
List[Atom("add"), List[Atom("mul"), Atom("2"), Atom("3")], Atom("4")]
```

### 2. 型推論段階
```
Ann(
  List[
    Ann(Atom("add"), Fun[Int, Int, Int]),
    Ann(List[
      Ann(Atom("mul"), Fun[Int, Int, Int]),
      Ann(Atom("2"), Int),
      Ann(Atom("3"), Int)
    ], Int),
    Ann(Atom("4"), Int)
  ],
  Int
)
```

### 3. ハッシュ計算
```
add#a3f5c2b1: (Int Int -> Int)
mul#d7e9f3a2: (Int Int -> Int)
```

## パーサー実装方針

### レキサー（最小限）
- 空白/改行をトークン化（インデント計算用）
- アトム: 数値、文字列、識別子
- 括弧: ( )
- コメント: ;

### パーサー（再帰下降）
1. インデントレベルを追跡
2. 改行後のインデント増加 = リスト開始
3. インデント減少 = リスト終了
4. 同じインデント = 兄弟要素

## 利点

1. **極めてシンプル** - 3種類のノードのみ
2. **AIフレンドリー** - 木構造を直接操作
3. **人間も読み書き可能** - インデント構文
4. **型安全** - 推論結果をインライン化
5. **バージョン管理** - 関数レベルでハッシュ

## 移行戦略

1. 現在のASTから最小ASTへの変換器を実装
2. 最小パーサーを別モジュールとして実装
3. 型チェッカーを最小ASTに対応
4. 徐々に既存機能を最小構文で表現

## 例: 実際のコード

```x
module DataStructures v1.0.0

; リスト操作
def map f xs
  match xs
    [] -> []
    h :: t -> f h :: map f t

; 使用例
def double_all xs
  map \x -> * x 2 xs

; または
def double_all xs
  map (* 2) xs  ; 部分適用

; 型推論後
def map f xs : ((a -> b) List[a] -> List[b])
  match xs : List[a]
    [] -> [] : List[b]
    h :: t -> (f h : b) :: (map f t : List[b])
```