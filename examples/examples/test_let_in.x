module TestLetIn

let test_simple = fn () ->
  (let x = 5 in x + 10)

let test_nested = fn () ->
  (let x = 10 in
   let y = 20 in
   x + y)

let test_with_function = fn () ->
  (let add = fn a b -> a + b in
   let mul = fn a b -> a * b in
   add (mul 2 3) (mul 4 5))

let main = fn () ->
  test_simple () + test_nested () + test_with_function ()