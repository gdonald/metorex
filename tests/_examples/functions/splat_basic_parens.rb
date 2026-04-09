# Variadic parameter collects remaining args
def log(level, *messages)
  messages.each do |m|
    puts level + ": " + m
  end
end

log "INFO", "starting", "loading", "done"

# Splat with no extra args
def first_rest(first, *rest)
  puts first
  puts rest.length
end

first_rest "only"

# Splat expansion in call arguments
def add(a, b, c)
  a + b + c
end

args = [1, 2, 3]
puts add(*args)

# Splat with some fixed and some expanded
def greet(greeting, name)
  puts greeting + ", " + name
end

pair = ["Hi", "Alice"]
greet(*pair)
