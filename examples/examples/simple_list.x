module SimpleList

type List = Nil | Cons Int List

let length = (fn (list)
    (match list
        (Nil 0)
        ((Cons x xs) (+ 1 (length xs)))))

let map = (fn (f list)
    (match list
        (Nil Nil)
        ((Cons x xs) (Cons (f x) (map f xs)))))

test "simple list" {
    let list1 = (Cons 1 (Cons 2 (Cons 3 Nil)))
    let list2 = (map (fn (x) (* x 2)) list1)
    
    (&& (== (length list1) 3)
        (== (length list2) 3))
}