map :: (Int -> Int) -> [Int] -> [Int]
map f (x:xs) = (f x) : ((map f) xs)

