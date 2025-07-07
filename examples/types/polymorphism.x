module Polymorphism

# Polymorphic identity function
let id : forall a. a -> a = fn x -> x

# Polymorphic pair functions
let fst : forall a b. (a, b) -> a = fn p ->
  match p with
  | (x, _) -> x

let snd : forall a b. (a, b) -> b = fn p ->
  match p with
  | (_, y) -> y

# Higher-order polymorphic function
let compose : forall a b c. (b -> c) -> (a -> b) -> a -> c =
  fn f g x -> f (g x)

# Polymorphic data type
type Option a =
  | None
  | Some a

let map_option : forall a b. (a -> b) -> Option a -> Option b =
  fn f opt ->
    match opt with
    | None -> None
    | Some x -> Some (f x)

# Example usage
let main = fn () ->
  let x = id 42 in
  let s = id "hello" in
  let p = (x, s) in
  let first = fst p in
  let inc = fn x -> x + 1 in
  let double = fn x -> x * 2 in
  let inc_then_double = compose double inc in
  inc_then_double 5