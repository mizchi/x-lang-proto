module TestHash

// Test function with specific implementation
pub let add x y = x + y

// Another function that uses add
pub let double x = add x x

// Pure function
pub let multiply x y = x * y

// Function with dependencies
pub let calculate x y = 
    let sum = add x y in
    let prod = multiply x y in
    sum + prod

// Function with the same implementation as add
pub let plus a b = a + b