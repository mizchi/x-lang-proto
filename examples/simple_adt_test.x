module SimpleADTTest

data Option = None | Some Int

let unwrap = fun opt ->
    match opt with
    | None -> 0
    | Some x -> x

let main = fun () ->
    let x = Some 42 in
    let y = None in
    unwrap x