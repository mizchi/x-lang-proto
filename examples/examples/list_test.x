module ListTest

```
---
@function sum_list
@param {list: List} The list to sum
@returns {Int} Sum of all elements
---

Sums all elements in a list.
```
let sum_list = fn list ->
    match list with
    | [] -> 0
    | x :: xs -> x + sum_list xs

```
---
@function double_list
@param {list: List} The list to double
@returns {List} New list with doubled elements
---

Doubles all elements in a list.
```
let double_list = fn list ->
    match list with
    | [] -> []
    | x :: xs -> (x * 2) :: double_list xs

```
---
@test "list literal test"
---

Tests list literal syntax.
```
test "list literal test" {
    let list1 = [1; 2; 3; 4; 5] in
    let list2 = [10; 20; 30] in
    let empty = [] in
    sum_list list1 == 15 &&
    sum_list list2 == 60 &&
    sum_list empty == 0 &&
    double_list [1; 2; 3] == [2; 4; 6] &&
    double_list [] == []
}