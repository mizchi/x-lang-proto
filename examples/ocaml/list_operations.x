module ListOperations

(* List manipulation functions *)

let rec length = fun lst ->
  match lst with
  | [] -> 0
  | h :: t -> 1 + length t

let rec map = fun f lst ->
  match lst with
  | [] -> []
  | h :: t -> f h :: map f t

let rec filter = fun pred lst ->
  match lst with
  | [] -> []
  | h :: t ->
    if pred h then h :: filter pred t
    else filter pred t

let rec fold_left = fun f acc lst ->
  match lst with
  | [] -> acc
  | h :: t -> fold_left f (f acc h) t

(* Example usage *)
let double = fun x -> x * 2
let is_even = fun x -> x mod 2 = 0

let main = fun () ->
  let numbers = [1; 2; 3; 4; 5] in
  let doubled = map double numbers in
  let evens = filter is_even numbers in
  let sum = fold_left (fun acc x -> acc + x) 0 numbers in
  sum