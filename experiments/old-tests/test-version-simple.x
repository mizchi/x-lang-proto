module TestVersionSimple

-- Basic addition function
let add = fn x y -> x + y

-- String concatenation (different type signature)
let concat = fn x y -> x ++ y

-- Addition with side effect
let add_with_print = fn x y ->
    -- This would have IO effect
    let result = x + y in
    result