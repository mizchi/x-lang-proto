# do notation examples for effectful computations

# Basic do notation with IO effect
printGreeting : Text ->{IO} ()
printGreeting name = do
  print "Hello,"
  print name
  print "!"

# do notation with bind (monadic style)
readAndGreet : () ->{IO} ()
readAndGreet = do
  print "What's your name?"
  name <- readLine
  print ("Hello, " ++ name ++ "!")

# Multiple effects
processFile : Text ->{IO, Error} Text
processFile filename = do
  handle <- openFile filename
  contents <- readFile handle
  closeFile handle
  return contents

# do notation with pattern matching
parseAndProcess : Text ->{Error} Int
parseAndProcess input = do
  match parseInt input with
    Some n -> return (n * 2)
    None -> throw "Invalid number"

# Combining pipelines with do notation
transformData : List Int ->{Async} List Int
transformData data = do
  filtered <- async (filter positive data)
  mapped <- async (map double filtered)
  return mapped

# State effect example
counter : () ->{State Int} Int
counter = do
  current <- get
  put (current + 1)
  get

# do notation with let bindings
complexCalculation : Int -> Int ->{Error} Int
complexCalculation x = do
  let y = x * 2
      z = y + 10
  if z > 100
    then throw "Result too large"
    else return z

# Nested do blocks
nestedEffects : () ->{IO} ()
nestedEffects = do
  print "Outer effect"
  result <- do
    print "Inner effect"
    return 42
  print ("Result: " ++ show result)

# do notation in pipelines
processWithEffects : List Text ->{IO} List Int
processWithEffects items =
  items
    |> traverse (\item -> do
         print ("Processing: " ++ item)
         return (length item))

# Error handling in do notation
safeDiv : Int -> Int ->{Error} Int
safeDiv x y = do
  if y == 0
    then throw "Division by zero"
    else return (x / y)

# Async operations
fetchData : Text ->{Async, Error} Data
fetchData url = do
  response <- httpGet url
  match response with
    Ok data -> return (parseData data)
    Err msg -> throw ("Failed to fetch: " ++ msg)

# Combining multiple effects with handlers
runWithHandlers : () -> Int
runWithHandlers = 
  handle
    handle
      do
        x <- get
        put (x + 1)
        y <- get
        return y
    with State 0 s ->
      | get k -> k s s
      | put s' k -> k () s'
  with Error ->
    | throw msg k -> 0
    | return x -> x