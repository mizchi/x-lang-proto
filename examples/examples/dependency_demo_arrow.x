module DependencyDemo

let add = (fn (x y) (+ x y))

let multiply = (fn (x y) (* x y))

let double = (fn (x) (multiply x 2))

let quadruple = (fn (x) (double (double x)))

let calculate = (fn (x y) (add (double x) (multiply y 3)))

let factorial = (fn (n) 
  (if (<= n 1)
      1 
      (multiply n (factorial (- n 1)))))

let main = (fn () 
  (let ((result (calculate 5 10)))
    (let ((fact (factorial 5)))
      (+ result fact))))