module ListLiteralTest

let test_list_literal = fn () ->
    let list1 = [1; 2; 3] in
    let list2 = [4; 5; 6] in
    let empty = [] in
    list1

let test_cons = fn () ->
    let list = 1 :: 2 :: 3 :: [] in
    list