module TestHashCorrect

-- Test function with specific implementation
let add = fun x y -> x + y

-- Another function that uses add
let double = fun x -> add x x

-- Pure function
let multiply = fun x y -> x * y

-- Function with dependencies
let calculate = fun x y -> 
    let sum = add x y in
    let prod = multiply x y in
    sum + prod

-- Function with the same implementation as add
let plus = fun a b -> a + b