# Hash writes and reads with String and Symbol keys.
by_name = {}
50_000.times { |step| by_name["key#{step}"] = step }
total = 0
50_000.times { |step| total += by_name["key#{step}"] }
by_symbol = { alpha: 1, beta: 2, gamma: 3 }
200_000.times { total += by_symbol[:beta] }
puts total
