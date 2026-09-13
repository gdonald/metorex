puts([3].map { it * 2 }.inspect)
puts(-> { it + -> { it * it }.call(2) }.call(3))
puts(-> { it; binding.local_variables }.call("a").inspect)
puts(-> { _1 + _2; binding.local_variables }.call(1, 2).inspect)

it = 0
puts(proc { it }.call("a"))

refused = [
  "proc { |x| it }",
  "-> () { it }",
  "proc { it + _1 }",
  "proc { _1 + it }",
  "proc { _1 = 0 }",
  "proc { |x| _1 }"
]
refused.each do |source|
  begin
    eval(source)
    puts("no error for #{source}")
  rescue SyntaxError => error
    puts(error.message.split(": ").last)
  end
end
