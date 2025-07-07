module DependencyDemo

let add = fn x -> fn y -> x + y

let multiply = fn x -> fn y -> x * y

let double = fn x -> multiply x 2

let quadruple = fn x -> double (double x)

let calculate = fn x -> fn y -> add (double x) (multiply y 3)

let main = fn () -> 
  let result = calculate 5 10 in
  let fact = 120 in
  result + fact