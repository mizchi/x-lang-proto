module TestLetIn

let test_simple = (fn ()
  (let ((x 5)) (+ x 10)))

let test_nested = (fn ()
  (let ((x 10))
   (let ((y 20))
   (+ x y))))

let test_with_function = (fn ()
  (let ((add (fn (a b) (+ a b))))
   (let ((mul (fn (a b) (* a b))))
   (add (mul 2 3) (mul 4 5)))))

let main = (fn ()
  (+ (+ (test_simple) (test_nested)) (test_with_function)))