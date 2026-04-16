h = {}
h[nil] = "n"
h[true] = "t"
h[false] = "f"
h[:sym] = "s"
h[42] = "i"
h[3.5] = "fl"
h["str"] = "st"

puts h.keys.length
puts h.values.length
puts h[nil]
puts h[true]
puts h[false]
puts h[:sym]
puts h[42]
puts h[3.5]
puts h["str"]
