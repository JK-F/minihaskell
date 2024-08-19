comp f [] = []
comp f (x:xs) = if (f x) then x : (comp (f) xs) else comp (f) xs

notdiv x y = (y `mod` x) /= 0

sieve [] = []
sieve (x:xs) = x : (sieve (comp (notdiv x) (xs)))

first 0 _ = []
first n (x:xs) = x : (first (n-1) xs)

head (x:xs) = x

range = [2..]
primes = sieve range

x = first 5 primes

x
