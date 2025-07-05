module Main =
  let factorial = fun n ->
    if n <= 1 then 1
    else n * factorial (n - 1)

  let main = 
    let result = factorial 5 in
    print result