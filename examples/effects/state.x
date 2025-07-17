module StateExample

# Define State effect
effect State {
  get : (-> () int)
  put : (-> int ())
}

# Stateful counter
let counter = (fn () 
  (let ((x (perform (State.get ()))))
    (perform (State.put (+ x 1)))
    x))

# Handler for State effect
let run_state = (fn (initial action)
  ((handle (action ())
    ((return x) (fn (s) (pair x s)))
    ((State.get () resume) (fn (s) ((resume s) s)))
    ((State.put s' resume) (fn (_) ((resume ()) s'))))
   initial))

# Example usage
let main = (fn ()
  (let ((increment_twice (fn ()
                          (let ((a (counter ()))
                                (b (counter ())))
                            (pair a b)))))
    (run_state 0 increment_twice)))