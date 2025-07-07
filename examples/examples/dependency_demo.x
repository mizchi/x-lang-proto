module DependencyDemo where

# Basic arithmetic functions
add :: Int -> Int -> Int
add x y = x + y

multiply :: Int -> Int -> Int
multiply x y = x * y

# Functions with dependencies
double :: Int -> Int
double x = multiply x 2

# More complex dependency
quadruple :: Int -> Int
quadruple x = double (double x)

# Function with multiple dependencies
calculate :: Int -> Int -> Int
calculate x y = add (double x) (multiply y 3)

# Recursive function
factorial :: Int -> Int
factorial n = 
  if n <= 1 
  then 1 
  else multiply n (factorial (n - 1))

# Mutually recursive functions
isEven :: Int -> Bool
isEven n = 
  if n == 0 
  then True 
  else isOdd (n - 1)

isOdd :: Int -> Bool
isOdd n = 
  if n == 0 
  then False 
  else isEven (n - 1)

# Entry point using multiple functions
main :: IO ()
main = do
  let result = calculate 5 10
  let fact5 = factorial 5
  print result
  print fact5