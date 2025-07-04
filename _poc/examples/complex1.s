(module math
  (export factorial fibonacci)
  
  (defun factorial (n)
    (if (= n 0)
        1
        (* n (factorial (- n 1)))))
  
  (defun fibonacci (n)
    (if (< n 2)
        n
        (+ (fibonacci (- n 1))
           (fibonacci (- n 2)))))
  
  (defstruct point
    (x 0)
    (y 0)))