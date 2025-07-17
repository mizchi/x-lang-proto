module Polymorphism

# Polymorphic identity function
let id : (forall a. (-> a a)) = (fn (x) x)

# Polymorphic pair functions
let fst : (forall a b. (-> (pair a b) a)) = (fn (p)
  (match p
    ((pair x _) x)))

let snd : (forall a b. (-> (pair a b) b)) = (fn (p)
  (match p
    ((pair _ y) y)))

# Higher-order polymorphic function
let compose : (forall a b c. (-> (-> b c) (-> a b) (-> a c))) =
  (fn (f g x) (f (g x)))

# Polymorphic data type
type Option a =
  | None
  | Some a

let map_option : (forall a b. (-> (-> a b) (-> (Option a) (Option b)))) =
  (fn (f opt)
    (match opt
      (None None)
      ((Some x) (Some (f x)))))

# Example usage
let main = (fn ()
  (let ((x (id 42))
        (s (id "hello"))
        (p (pair x s))
        (first (fst p))
        (inc (fn (x) (+ x 1)))
        (double (fn (x) (* x 2)))
        (inc_then_double (compose double inc)))
    (inc_then_double 5)))