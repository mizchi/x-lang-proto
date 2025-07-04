(module math
  (export factorial fibonacci distance)
  
  (defun factorial (n)
    (if (<= n 1)
        1
        (* n (factorial (- n 1)))))
  
  (defun fibonacci (n)
    (if (< n 2)
        n
        (+ (fibonacci (- n 1))
           (fibonacci (- n 2)))))
  
  (defstruct point
    (x 0)
    (y 0)
    (z 0))
  
  (defun distance (p1 p2)
    (sqrt (+ (square (- (point-x p2) (point-x p1)))
             (square (- (point-y p2) (point-y p1)))
             (square (- (point-z p2) (point-z p1)))))))