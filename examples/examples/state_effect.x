module StateEffect

```
---
@effect State
@typeParam s The type of the state
@operations [get, put]
---

State effect for stateful computations.
```
effect State[s] {
    get : Unit -> s
    put : s -> Unit
}

```
---
@function run_state
@param {initial: s} Initial state value
@param {computation: Unit -> a with State[s]} Stateful computation
@returns {(a, s)} Result and final state
---

Runs a stateful computation with an initial state.
```
let run_state = fn initial computation ->
    let state_ref = ref initial in
    handle computation () with
    | get () -> resume (!state_ref)
    | put new_state -> 
        state_ref := new_state;
        resume ()
    | return x -> (x, !state_ref)

```
---
@function modify
@param {f: s -> s} Function to modify the state
@returns {Unit} Unit value
@effect State[s]
---

Modifies the state using a function.
```
let modify = fn f ->
    let current = perform get () in
    perform put (f current)

```
---
@function increment
@returns {Unit} Unit value
@effect State[Int]
---

Increments an integer state by 1.
```
let increment = fn () ->
    modify (fn x -> x + 1)

```
---
@function decrement
@returns {Unit} Unit value  
@effect State[Int]
---

Decrements an integer state by 1.
```
let decrement = fn () ->
    modify (fn x -> x - 1)

```
---
@function counter_example
@returns {Int} Final counter value
@effect State[Int]
---

Example of using state effect for a counter.
```
let counter_example = fn () ->
    do {
        increment ();
        increment ();
        let x = perform get () in
        increment ();
        decrement ();
        let y = perform get () in
        return x + y
    }

```
---
@function fibonacci_stateful
@param {n: Int} The position in sequence
@returns {Int} The nth Fibonacci number
@effect State[(Int, Int)]
---

Fibonacci using state to track last two values.
```
let fibonacci_stateful = fn n ->
    if n <= 1 then
        n
    else
        let computation = fn () ->
            let rec fib_helper = fn i ->
                if i >= n then
                    let (a, b) = perform get () in
                    b
                else
                    do {
                        let (a, b) = perform get () in
                        perform put (b, a + b);
                        fib_helper (i + 1)
                    }
            in
            fib_helper 2
        in
        let (result, _) = run_state (0, 1) computation in
        result

```
---
@test "state effect"
@tags ["unit", "effect", "state"]
---

Tests state effect operations.
```
test "state effect" {
    # Test basic state operations
    let (result1, final1) = run_state 0 (fn () ->
        do {
            increment ();
            increment ();
            let x = perform get () in
            return x
        }
    ) in
    result1 == 2 &&
    final1 == 2 &&
    
    # Test counter example
    let (result2, final2) = run_state 0 counter_example in
    result2 == 5 &&  # 2 + 3
    final2 == 2 &&
    
    # Test stateful fibonacci
    fibonacci_stateful 0 == 0 &&
    fibonacci_stateful 1 == 1 &&
    fibonacci_stateful 2 == 1 &&
    fibonacci_stateful 3 == 2 &&
    fibonacci_stateful 4 == 3 &&
    fibonacci_stateful 5 == 5 &&
    fibonacci_stateful 10 == 55
}