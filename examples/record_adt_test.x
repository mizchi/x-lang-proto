module RecordADTTest

data Person = 
    | Student { name: String, age: Int, grade: Int }
    | Teacher { name: String, age: Int, subject: String }
    | Staff { name: String, age: Int }

let get_name = fun person ->
    match person with
    | Student { name, age, grade } -> name
    | Teacher { name, age, subject } -> name  
    | Staff { name, age } -> name

let alice = Student { name = "Alice", age = 20, grade = 2 }
let bob = Teacher { name = "Bob", age = 35, subject = "Math" }
let charlie = Staff { name = "Charlie", age = 45 }

let main = fun () -> 42