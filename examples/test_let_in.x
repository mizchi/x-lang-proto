module TestLetIn

let test_simple = fun () ->
  (let x = 5 in x + 10)

let test_nested = fun () ->
  (let x = 10 in
   let y = 20 in
   x + y)

let test_with_function = fun () ->
  (let add = fun a b -> a + b in
   let mul = fun a b -> a * b in
   add (mul 2 3) (mul 4 5))

let main = fun () ->
  test_simple () + test_nested () + test_with_function ()