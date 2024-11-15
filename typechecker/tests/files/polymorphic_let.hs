fst (x, _) = x

let id = \x -> x in (id 3, id True)

\x -> let f1 = \z -> (fst x, z) in (fst (f1 3) 42, fst (f1 True) 17)

let const = \x -> \y -> x in (const 3 True, const False 4)

\x -> let x1 = fst x in x1 3

