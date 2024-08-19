comp f [] = []
comp f (x:xs) = if (f x) then x : (comp (f) xs) else comp (f) xs

filter x = x > 2

comp (filter) ([1, 2, 3, 4, 5, 6, 7, 8])
