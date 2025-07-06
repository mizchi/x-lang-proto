module BinaryTree

```
---
@type Tree
@typeParam a The type of elements in the tree
---

Binary search tree data structure.
```
data Tree[a] = 
    | Empty
    | Node { value: a, left: Tree[a], right: Tree[a] }

```
---
@function insert
@param {x: a} Element to insert
@param {tree: Tree[a]} The tree to insert into
@returns {Tree[a]} New tree with element inserted
@complexity O(log n) average, O(n) worst case
---

Inserts an element into a binary search tree.
```
let insert = fun x tree ->
    match tree with
    | Empty -> Node { value = x, left = Empty, right = Empty }
    | Node { value, left, right } ->
        if x < value then
            Node { value = value, left = insert x left, right = right }
        else if x > value then
            Node { value = value, left = left, right = insert x right }
        else
            tree  // Element already exists

```
---
@function contains
@param {x: a} Element to search for
@param {tree: Tree[a]} The tree to search in
@returns {Bool} True if element exists in tree
@complexity O(log n) average, O(n) worst case
---

Checks if an element exists in the tree.
```
let contains = fun x tree ->
    match tree with
    | Empty -> false
    | Node { value, left, right } ->
        if x == value then
            true
        else if x < value then
            contains x left
        else
            contains x right

```
---
@function min_value
@param {tree: Tree[a]} Non-empty tree
@returns {a} The minimum value in the tree
@throws "Empty tree" if tree is empty
---

Finds the minimum value in a tree.
```
let min_value = fun tree ->
    match tree with
    | Empty -> error "min_value: empty tree"
    | Node { value, left = Empty, right = _ } -> value
    | Node { value = _, left, right = _ } -> min_value left

```
---
@function delete
@param {x: a} Element to delete
@param {tree: Tree[a]} The tree to delete from
@returns {Tree[a]} New tree with element deleted
@complexity O(log n) average, O(n) worst case
---

Deletes an element from the tree.
```
let delete = fun x tree ->
    match tree with
    | Empty -> Empty
    | Node { value, left, right } ->
        if x < value then
            Node { value = value, left = delete x left, right = right }
        else if x > value then
            Node { value = value, left = left, right = delete x right }
        else
            // Found the node to delete
            match (left, right) with
            | (Empty, Empty) -> Empty
            | (Empty, r) -> r
            | (l, Empty) -> l
            | (l, r) ->
                let successor = min_value r in
                Node { value = successor, left = l, right = delete successor r }

```
---
@function inorder
@param {tree: Tree[a]} The tree to traverse
@returns {List[a]} List of elements in sorted order
@complexity O(n)
---

Performs inorder traversal returning a sorted list.
```
let inorder = fun tree ->
    match tree with
    | Empty -> []
    | Node { value, left, right } ->
        append (inorder left) (value :: inorder right))

```
---
@function height
@param {tree: Tree[a]} The tree to measure
@returns {Int} Height of the tree
@complexity O(n)
---

Calculates the height of a tree.
```
let height = fun tree ->
    match tree with
    | Empty -> 0
    | Node { value = _, left, right } ->
        1 + max (height left) (height right)

```
---
@function is_balanced
@param {tree: Tree[a]} The tree to check
@returns {Bool} True if tree is balanced
@complexity O(n)
---

Checks if a tree is balanced (height difference <= 1).
```
let is_balanced = fun tree ->
    let rec check = fun t ->
        match t with
        | Empty -> (true, 0)
        | Node { value = _, left, right } ->
            let (left_balanced, left_height) = check left in
            let (right_balanced, right_height) = check right in
            let balanced = left_balanced && right_balanced && 
                          abs (left_height - right_height) <= 1 in
            (balanced, 1 + max left_height right_height)
    in
    let (balanced, _) = check tree in
    balanced

```
---
@test "binary tree operations"
@tags ["unit", "tree", "data-structure"]
---

Tests binary search tree operations.
```
test "binary tree operations" {
    let empty = Empty
    let tree1 = insert 5 empty
    let tree2 = insert 3 tree1
    let tree3 = insert 7 tree2
    let tree4 = insert 1 tree3
    let tree5 = insert 9 tree4
    
    // Test contains
    contains 5 tree5 &&
    contains 3 tree5 &&
    contains 7 tree5 &&
    not (contains 4 tree5) &&
    not (contains 0 empty) &&
    
    // Test min_value
    min_value tree5 == 1 &&
    
    // Test inorder traversal
    inorder tree5 == [1, 3, 5, 7, 9] &&
    inorder empty == [] &&
    
    // Test delete
    let tree_del = delete 3 tree5 in
    not (contains 3 tree_del) &&
    contains 5 tree_del &&
    contains 1 tree_del &&
    inorder tree_del == [1, 5, 7, 9] &&
    
    // Test height
    height empty == 0 &&
    height tree5 >= 3 &&
    
    // Test balanced tree
    let balanced = insert 4 (insert 2 (insert 6 tree3)) in
    is_balanced balanced
}