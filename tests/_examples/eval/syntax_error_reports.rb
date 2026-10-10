# Code `eval` refuses raises a SyntaxError laid out as prism lays it out:
# each line at fault marked with `>`, the lines around it, and a run of
# carets under the part at fault with what is wrong there.
def report(source, *where)
  eval(source, binding, *where)
rescue SyntaxError => error
  puts(error.message.sub(/\A\(eval at [^)]*\)/, "(eval)"))
end

report("proc { _1 = 0 }")
report("x = 1 +")
report("first\nsecond\nthird\nvalue = = 1\nfourth\nfifth")
report("total = 'a long string that runs on past the caret' + + ) + 1")
report("ä = 1 +")
report("1 +", "named.rb", 10)
