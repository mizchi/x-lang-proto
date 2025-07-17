module Math

let add = (fn (x y) (+ x y))

let factorial = (fn (n)
  (if (<= n 1) 
      1
      (* n (factorial (- n 1)))))

let main = (fn ()
  (+ (add 5 3) (factorial 5)))