module TestBuiltin

let add_numbers = (fn (x y) (+ x y))

let multiply_numbers = (fn (x y) (* x y))

let test_arithmetic = (fn ()
  (+ (add_numbers 1 2) (multiply_numbers 3 4)))

let test_comparison = (fn ()
  (< 5 10))

let test_bool = (fn ()
  (|| (&& true false) true))

let main = (fn ()
  (print_endline "Hello from x Language!"))