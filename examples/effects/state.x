module StateExample

(* Define State effect *)
effect State {
  get : () -> int
  put : int -> ()
}

(* Stateful counter *)
let counter = fun () ->
  let x = perform State.get () in
  perform State.put (x + 1);
  x

(* Handler for State effect *)
let run_state = fun initial action ->
  handle action () with
  | return x -> fun s -> (x, s)
  | State.get () resume -> fun s -> resume s s
  | State.put s' resume -> fun _ -> resume () s'
  end initial

(* Example usage *)
let main = fun () ->
  let increment_twice = fun () ->
    let a = counter () in
    let b = counter () in
    (a, b)
  in
  run_state 0 increment_twice