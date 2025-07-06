module DependencyDemo

let add = fun x -> fun y -> x + y

let multiply = fun x -> fun y -> x * y

let double = fun x -> multiply x 2

let quadruple = fun x -> double (double x)

let calculate = fun x -> fun y -> add (double x) (multiply y 3)

let main = fun () -> 
  let result = calculate 5 10 in
  let fact = 120 in
  result + fact