def with_rescue
  raise "oops"
rescue
  puts "rescued"
end

with_rescue()

def with_ensure
  puts "body"
ensure
  puts "ensure ran"
end

with_ensure()

def with_both
  raise "fail"
rescue
  puts "caught"
ensure
  puts "cleanup"
end

with_both()
