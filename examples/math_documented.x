module MathDocumented

```
---
@function gcd
@param {a: Int} First number
@param {b: Int} Second number
@returns {Int} Greatest common divisor
@algorithm "Euclidean"
---

Calculates the greatest common divisor using Euclidean algorithm.
```
let gcd = fun a b ->
    if b == 0 then a else gcd b (a % b)

```
---
@function lcm  
@param {a: Int} First number
@param {b: Int} Second number
@returns {Int} Least common multiple
---

Calculates the least common multiple.
```
let lcm = fun a b ->
    (a * b) / gcd a b