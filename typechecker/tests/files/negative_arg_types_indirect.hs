fst (x, _) = x

\x -> let f1 = \z -> (fst x, z) in (fst (f1 3) 42, fst (f1 True) True)
