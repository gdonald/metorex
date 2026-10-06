# `&.` reaches `[]`, `[]=` and `.()` written as method names, and answers
# nil without calling them when the receiver is nil.
prices = { tea: 3 }
p(prices&.[](:tea))
p(prices&.[]=(:coffee, 4))
p(prices)

double = ->(amount) { amount * 2 }
p(double&.(5))

missing = nil
p(missing&.[](:tea))
p(missing&.[]=(:tea, 1))
p(missing&.(5))
p(missing&.[](:tea) || :none)
