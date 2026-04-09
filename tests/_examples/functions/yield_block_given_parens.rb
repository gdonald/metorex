def maybe_yield(x)
  if block_given?
    yield x
  else
    puts "no block: " + x.to_s
  end
end

maybe_yield 42 do |n|
  puts "got: " + n.to_s
end

maybe_yield 99
