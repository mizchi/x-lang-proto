module TestSimpleDeps

let identity = (fn (x) x)

let double = (fn (x) (add x x))

let add = (fn (a b) (+ a b))