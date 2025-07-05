module TestBuiltin

let add_numbers = fun x y -> x + y

let multiply_numbers = fun x y -> x * y

let test_arithmetic = fun () ->
  add_numbers 1 2 + multiply_numbers 3 4

let test_comparison = fun () ->
  5 < 10

let test_bool = fun () ->
  true && false || true

let main = fun () ->
  print_endline "Hello from x Language!"