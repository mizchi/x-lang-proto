module Math

let add = fun x y -> x + y

let factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)

let main = fun () ->
  add 5 3 + factorial 5