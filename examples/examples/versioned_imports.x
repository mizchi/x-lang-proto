module VersionedImports

# Import specific versions of functions
# import Math.add@^1.0.0
# import Math.multiply@=2.0.0
# import Utils.format@latest

# For now, demonstrate with comments how it would work
# When add@1.x is imported, any 1.x version is acceptable
# When multiply@=2.0.0 is imported, only exact version 2.0.0 is used

# Function that depends on specific versions
let calculate = fn x y ->
    # This would use add@^1.0.0
    let sum = add x y in
    # This would use multiply@=2.0.0  
    let product = multiply x y in
    sum + product

# Version constraints in function-level imports
# let processData =
#     import List.map@^2.0.0
#     import List.filter@^2.0.0
#     import String.format@^1.5.0
#     fn data ->
#         data
#         |> filter (fn x -> x > 0)
#         |> map (fn x -> format "Value: {}" x)

# Demonstrate version migration scenario
# v1.0.0: add takes 2 parameters
# v2.0.0: add takes 3 parameters (breaking change)
# Code using add@^1.0.0 continues to work with v1.x
# Code must be updated to use add@^2.0.0 for new version