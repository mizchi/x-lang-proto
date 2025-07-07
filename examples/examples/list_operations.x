module ListOperations

```
#
@fnction length
@param {list: List[a]} The list to measure
@returns {Int} The length of the list
@complexity O(n)
#

Calculates the length of a list recursively.
```
let length = fn list ->
    match list with
    | [] -> 0
    | _ :: xs -> 1 + length xs

```
#
@fnction map
@param {f: a -> b} Function to apply to each element
@param {list: List[a]} The input list
@returns {List[b]} A new list with f applied to each element
@complexity O(n)
#

Maps a fnction over a list.
```
let map = fn f list ->
    match list with
    | [] -> []
    | x::xs -> (f x) :: (map f xs)

```
#
@fnction filter
@param {pred: a -> Bool} Predicate fnction
@param {list: List[a]} The input list
@returns {List[a]} List containing only elements that satisfy the predicate
@complexity O(n)
#

Filters a list based on a predicate.
```
let filter = fn pred list ->
    match list with
    | [] -> []
    | x::xs -> 
        if pred x then
            x :: (filter pred xs)
        else
            filter pred xs

```
#
@fnction fold_left
@param {f: b -> a -> b} The folding fnction
@param {init: b} Initial accumulator value
@param {list: List[a]} The list to fold
@returns {b} The final accumulated value
@complexity O(n)
#

Left fold (reduce) over a list.
```
let fold_left = fn f init list ->
    match list with
    | [] -> init
    | x::xs -> fold_left f (f init x) xs

```
#
@fnction reverse
@param {list: List[a]} The list to reverse
@returns {List[a]} The reversed list
@complexity O(n)
#

Reverses a list efficiently using an accumulator.
```
let reverse = fn list ->
    let rev_helper = fn acc xs ->
        match xs with
        | [] -> acc
        | y :: ys -> rev_helper (y :: acc) ys
    in
    rev_helper [] list

```
#
@fnction append
@param {list1: List[a]} First list
@param {list2: List[a]} Second list
@returns {List[a]} Concatenation of list1 and list2
@complexity O(n) where n is length of list1
#

Appends two lists.
```
let append = fn list1 list2 ->
    match list1 with
    | [] -> list2
    | x::xs -> x :: (append xs list2)

```
#
@fnction flatten
@param {lists: List[List[a]]} List of lists
@returns {List[a]} Single flattened list
@complexity O(n*m) where n is number of lists and m is average length
#

Flattens a list of lists into a single list.
```
let flatten = fn lists ->
    fold_left append [] lists

```
#
@test "list operations"
@tags ["unit", "list"]
#

Tests various list operations.
```
test "list operations" {
    let nums = [1; 2; 3; 4; 5]
    let empty = []
    
    # Test length
    length nums == 5 &&
    length empty == 0 &&
    
    # Test map
    let doubled = map (fn x -> x * 2) nums in
    let expected_doubled = [2; 4; 6; 8; 10] in
    doubled == expected_doubled &&
    map (fn x -> x + 1) empty == [] &&
    
    # Test filter
    let filtered = filter (fn x -> x > 3) nums in
    let expected_filtered = [4; 5] in
    filtered == expected_filtered &&
    filter (fn x -> x < 0) nums == [] &&
    
    # Test fold_left
    fold_left (fn acc x -> acc + x) 0 nums == 15 &&
    let reversed_via_fold = fold_left (fn acc x -> x :: acc) [] nums in
    let expected_reversed = [5; 4; 3; 2; 1] in
    reversed_via_fold == expected_reversed &&
    
    # Test reverse
    reverse nums == expected_reversed &&
    reverse empty == [] &&
    
    # Test append
    let list1 = [1; 2] in
    let list2 = [3; 4] in
    let appended = append list1 list2 in
    let expected_appended = [1; 2; 3; 4] in
    appended == expected_appended &&
    append [] nums == nums &&
    
    # Test flatten
    let list_of_lists = [[1; 2]; [3; 4]; [5]] in
    let flattened = flatten list_of_lists in
    flattened == nums &&
    flatten [[]] == []
}