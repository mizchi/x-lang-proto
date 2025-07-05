module Polymorphism

(* Polymorphic identity function *)
let id : forall a. a -> a = fun x -> x

(* Polymorphic pair functions *)
let fst : forall a b. (a, b) -> a = fun p ->
  match p with
  | (x, _) -> x

let snd : forall a b. (a, b) -> b = fun p ->
  match p with
  | (_, y) -> y

(* Higher-order polymorphic function *)
let compose : forall a b c. (b -> c) -> (a -> b) -> a -> c =
  fun f g x -> f (g x)

(* Polymorphic data type *)
type Option a =
  | None
  | Some a

let map_option : forall a b. (a -> b) -> Option a -> Option b =
  fun f opt ->
    match opt with
    | None -> None
    | Some x -> Some (f x)

(* Example usage *)
let main = fun () ->
  let x = id 42 in
  let s = id "hello" in
  let p = (x, s) in
  let first = fst p in
  let inc = fun x -> x + 1 in
  let double = fun x -> x * 2 in
  let inc_then_double = compose double inc in
  inc_then_double 5