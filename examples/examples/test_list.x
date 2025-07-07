module TestList

let test_cons = fn () ->
  1 :: 2 :: 3 :: []

let test_list_literal = fn () ->
  [1; 2; 3; 4; 5]

let main = fn () ->
  test_list_literal ()