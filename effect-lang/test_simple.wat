;; Simple test WAT file that should compile with wasmtime
(module
  (func $add (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.add
  )
  
  (func $main (result i32)
    i32.const 10
    i32.const 32
    call $add
  )
  
  (export "main" (func $main))
)