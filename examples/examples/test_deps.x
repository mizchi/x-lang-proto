module TestDeps

let add = (fn (x y) (+ x y))

let double = (fn (x) (add x x))

let quadruple = (fn (x) (double (double x)))

let main = (fn () (quadruple 5))