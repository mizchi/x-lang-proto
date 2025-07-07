# Examples demonstrating equivalent indent and brace syntax

# 1. Simple function - three equivalent ways
# Indent style
add1 x y =
  x + y

# Brace style
add2 x y = {
  x + y
}

# One-liner
add3 x y = { x + y }

# 2. Multi-statement function
# Indent style
calculate1 x y =
  let a = x * 2
      b = y * 3
  in
    a + b

# Brace style with semicolons
calculate2 x y = {
  let a = x * 2;
      b = y * 3;
  in
    a + b
}

# More explicit braces
calculate3 x y = {
  let {
    a = x * 2;
    b = y * 3;
  };
  in { a + b }
}

# 3. Pipeline operations
# Indent style
process1 data =
  data
    |> filter positive
    |> map double
    |> reduce 0 (+)

# Brace style
process2 data = {
  data
    |> filter positive
    |> map double
    |> reduce 0 (+)
}

# Mixed semicolons
process3 data = {
  data |> filter positive;
       |> map double;
       |> reduce 0 (+)
}

# 4. Conditional expressions
# Indent style
abs1 x =
  if x < 0
    then -x
    else x

# Brace style
abs2 x = {
  if x < 0 {
    then -x
  } else {
    x
  }
}

# Mixed style
abs3 x =
  if x < 0 { then -x }
  else x

# 5. Pattern matching
# Indent style
factorial1 n =
  match n with
    0 -> 1
    n -> n * factorial1 (n - 1)

# Brace style
factorial2 n = {
  match n with {
    0 -> 1;
    n -> n * factorial2 (n - 1)
  }
}

# 6. Nested functions
# Indent style
outer1 x =
  let inner y =
        x + y
  in
    inner 10

# Brace style
outer2 x = {
  let inner y = {
    x + y
  };
  in
    inner 10
}

# 7. Complex mixed example
complexFunction data options =
  # Start with indent
  let config = 
        parseOptions options
      validator = 
        getValidator config
  in {
    # Switch to braces for pipeline
    data
      |> validate validator
      |> transform config;
      |> finalize
  }

# 8. Module-level definitions can also use braces
module Utils {
  export add;
  export multiply;
  
  add x y = { x + y };
  
  multiply x y = {
    x * y
  }
}

# 9. Type definitions with effects
processRemote : List a ->{Remote, Async} List b
processRemote items = {
  items |> mapRemote transform;
        |> filterAsync isValid
}

# 10. Demonstrating that newline = semicolon
equivalent1 x y =
  let a = x
      b = y
  in a + b

equivalent2 x y = {
  let a = x; b = y;
  in a + b
}

# Both produce the same result