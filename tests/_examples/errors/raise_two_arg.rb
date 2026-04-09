begin
  raise RuntimeError, "custom message"
rescue
  puts("caught two-arg raise")
end
