module TestList

let test_cons = fun () ->
  1 :: 2 :: 3 :: []

let test_list_literal = fun () ->
  [1; 2; 3; 4; 5]

let main = fun () ->
  test_list_literal ()