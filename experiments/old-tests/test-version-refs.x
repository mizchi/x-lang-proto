module TestVersionRefs

-- Reference specific version by hash (content-addressed)
-- let add_v1 = #0a384d71a5e7ac72801858f16f3f25a8a341961240a8262132707a8c731992a2

-- Reference by name with version constraint
-- import add@^1.0.0 as add_compat
-- import add@=1.0.0 as add_exact

-- For now, use regular definitions
let add = fn x y -> x + y

-- This uses whatever version of add is compatible
let calculate = fn a b c ->
    let sum1 = add a b in
    let sum2 = add sum1 c in
    sum2

-- Demonstrate version migration
-- When add changes from v1.0.0 to v1.1.0 (compatible),
-- calculate continues to work
-- When add changes to v2.0.0 (breaking), 
-- calculate would need to be updated or pinned to v1.x