module DependencyDemo

let add = (x, y) -> x + y

let multiply = (x, y) -> x * y

let double = (x) -> multiply x 2

let quadruple = (x) -> double (double x)

let calculate = (x, y) -> add (double x) (multiply y 3)

let factorial = (n) -> 
  if n <= 1 
  then 1 
  else multiply n (factorial (n - 1))

let main = () -> 
  let result = calculate 5 10 in
  let fact = factorial 5 in
  result + fact