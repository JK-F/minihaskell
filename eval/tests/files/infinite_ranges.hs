map _ [] = []
map f (x:xs) = (f x) : map f xs

nat = [1, 2..]

even = map \x -> 2 * x nat

odd = map (\x -> 2 * x - 1) nat
