module TestHashV2

-- Version 2.0.0 - Breaking change: Now takes 3 parameters
let add = fn x y z -> x + y + z

-- Version 1.1.0 - Compatible change: Better implementation
let double = fn x -> 
    let result = x + x in
    result