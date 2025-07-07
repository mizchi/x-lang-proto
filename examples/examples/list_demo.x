module ListDemo

let sum = fn list ->
    match list with
    | [] -> 0
    | x :: xs -> x + sum xs

let main = fn () ->
    let nums = [1; 2; 3; 4; 5] in
    let result = sum nums in
    result