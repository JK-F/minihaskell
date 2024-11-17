first _ [] = []
first 0 _ = []
first n (x:xs) = (x : first (n - 1) xs)

inf _ = inf 1 

list = [1, 2, 3, inf 0]

first 3 list

