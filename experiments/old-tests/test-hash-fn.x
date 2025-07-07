module TestHashFn

-- Test function with specific implementation
let add = fn x y -> x + y

-- Another function that uses add
let double = fn x -> add x x