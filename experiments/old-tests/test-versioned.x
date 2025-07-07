module TestVersioned

-- Version 1.0.0: Basic addition
let add_v1 : Int -> Int -> Int = 
    fn x y -> x + y

-- Version 1.1.0: Same signature (compatible)
let add_v11 : Int -> Int -> Int = 
    fn x y -> 
        -- Optimized implementation
        x + y

-- Version 2.0.0: Breaking change - different parameter type
let add_v2 : String -> String -> String = 
    fn x y -> x ++ y

-- Version 1.2.0: Forward compatible - added effect
let add_with_log : Int -> Int -> Int <IO> = 
    fn x y -> 
        -- perform IO.log "Adding numbers"
        x + y

-- Function that depends on specific version
let double_v1 : Int -> Int = 
    fn x -> add_v1 x x

-- Polymorphic version
let add_poly : forall a. a -> a -> a = 
    fn x y -> x