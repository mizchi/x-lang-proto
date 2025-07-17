module InterfaceExample

# Define an interface for collections
interface Collection {
  type t a
  
  val empty : (forall a. (t a))
  val add : (forall a. (-> a (t a) (t a)))
  val member : (forall a. (-> a (t a) bool))
  val size : (forall a. (-> (t a) int))
}

# Implementation for lists
module ListCollection : Collection {
  type t a = (list a)
  
  let empty = []
  
  let rec add = (fn (x lst) (:: x lst))
  
  let rec member = (fn (x lst)
    (match lst
      ([] false)
      ((:: h t) (if (= h x) true (member x t)))))
  
  let rec size = (fn (lst)
    (match lst
      ([] 0)
      ((:: _ t) (+ 1 (size t)))))
}

# Generic function using the interface
let add_all : (forall c a. (=> (Collection c) (-> (list a) (c.t a) (c.t a)))) =
  (fn (items collection)
    (match items
      ([] collection)
      ((:: h t) (add_all t (c.add h collection)))))

let main = (fn ()
  (let ((nums [1; 2; 3])
        (coll (ListCollection.add 0 ListCollection.empty))
        (result (add_all nums coll)))
    (ListCollection.size result)))