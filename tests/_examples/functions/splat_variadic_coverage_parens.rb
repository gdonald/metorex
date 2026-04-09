# Method with variadic + fixed params
class Logger
  def log(level, *messages)
    messages.each do |m|
      puts level + ": " + m
    end
  end
end

l = Logger.new
l.log "INFO", "start", "done"

# Standalone function with variadic
def join_all(sep, *parts)
  result = ""
  parts.each do |p|
    if result.length > 0
      result = result + sep
    end
    result = result + p.to_s
  end
  puts result
end

join_all "-", "a", "b", "c"

# Splat expansion in call
def add3(a, b, c)
  a + b + c
end

args = [10, 20, 30]
puts add3(*args)

# Empty variadic
def count_args(*args)
  puts args.length
end

count_args()
count_args 1, 2, 3
