a = [1,2,3]
a

f [] = 0
f (x:xs) = x

f [1, 2, 3]

inf x = inf x + 1

f [42, inf 0]


lecker (_, _, "Kuchen") = 1
lecker (_, _, "Lakritze") = 0

x = (1, inf 0, "Kuchen")

lecker x

three x = 3
three inf 1
