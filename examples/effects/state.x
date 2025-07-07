module StateExample

# Define State effect
effect State {
  get : () -> int
  put : int -> ()
}

# Stateful counter
let counter = fn () ->
  let x = perform State.get () in
  perform State.put (x + 1);
  x

# Handler for State effect
let run_state = fn initial action ->
  handle action () with
  | return x -> fn s -> (x, s)
  | State.get () resume -> fn s -> resume s s
  | State.put s' resume -> fn _ -> resume () s'
  end initial

# Example usage
let main = fn () ->
  let increment_twice = fn () ->
    let a = counter () in
    let b = counter () in
    (a, b)
  in
  run_state 0 increment_twice