inf x = inf x + 1

last (x:[]) = x
last (_:xs) = last xs

last [inf 1, inf 2, 3]
