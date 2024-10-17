multiple a b c = 1
y = 1
x = multiple 1
b = (x y) 3

double :: Int -> Int 
double x = 2 * x

triple :: Int -> Int
triple 1 = 3
triple x = triple (x - 1) + 3

zip (x:xs) (y:ys) = (x, y) : (zip xs ys)
