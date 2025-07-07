# Unison-style syntax examples

# Simple value definition
answer : Int
answer = 42

# Function with type signature
add : Int -> Int -> Int
add x y = x + y

# Pipeline example
distributed : List Int ->{Remote} Int
distributed dseq =
  dseq
    |> map (x -> x + 1)
    |> filter (x -> mod x 7 == 0)
    |> reduce 0 (+)

# Lambda expressions
inc : Int -> Int
inc = x -> x + 1

# Curried function
multiply : Int -> Int -> Int
multiply = x -> y -> x * y

# Pattern matching (future implementation)
factorial : Int -> Int
factorial n =
  if n == 0
    then 1
    else n * factorial (n - 1)

# List operations with pipelines
processData : List Int -> Int
processData data =
  data
    |> filter (x -> x > 0)
    |> map (x -> x * 2)
    |> fold 0 (+)

# Effect handlers (future implementation)
printAndReturn : Text ->{IO} Text
printAndReturn msg =
  print msg
  msg

# Higher-order functions
compose : (b -> c) -> (a -> b) -> a -> c
compose f g = x -> f (g x)

# Type aliases (future implementation)
type UserId = Int
type UserName = Text

# Data types (future implementation)
type User = User UserId UserName

# Effect definition (future implementation)
effect Store a where
  get : Text ->{Store a} Maybe a
  put : Text -> a ->{Store a} ()

# Module-level documentation
{- | This module demonstrates Unison-style syntax
     with pipelines, lambdas, and effects
-}

# Inline type annotations
annotatedValue : Int
annotatedValue = (42 : Int)