double :: Int -> Int 
double x = 2 * x

triple :: Int -> Int
triple 1 = 3
triple x = triple (x - 1) + 3
