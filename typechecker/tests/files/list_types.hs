zip :: [Int] -> [Int] -> [(Int, Int)]
zip (x:xs) (y:ys) = (x, y) : (zip xs ys)
