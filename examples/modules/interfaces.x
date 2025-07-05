module InterfaceExample

(* Define an interface for collections *)
interface Collection {
  type t a
  
  val empty : forall a. t a
  val add : forall a. a -> t a -> t a
  val member : forall a. a -> t a -> bool
  val size : forall a. t a -> int
}

(* Implementation for lists *)
module ListCollection : Collection {
  type t a = list a
  
  let empty = []
  
  let rec add = fun x lst -> x :: lst
  
  let rec member = fun x lst ->
    match lst with
    | [] -> false
    | h :: t -> if h = x then true else member x t
  
  let rec size = fun lst ->
    match lst with
    | [] -> 0
    | _ :: t -> 1 + size t
}

(* Generic function using the interface *)
let add_all : forall c a. Collection c => list a -> c.t a -> c.t a =
  fun items collection ->
    match items with
    | [] -> collection
    | h :: t -> add_all t (c.add h collection)

let main = fun () ->
  let nums = [1; 2; 3] in
  let coll = ListCollection.add 0 ListCollection.empty in
  let result = add_all nums coll in
  ListCollection.size result