# A range written with literal ends is built once, so the Strings at its ends
# are frozen. A range built from a variable keeps that String as it is.

letters = "a".."b"
p [letters.frozen?, letters.begin.frozen?, letters.end.frozen?]

start = +"c"
built = start.."d"
p [built.begin.frozen?, built.end.frozen?]
