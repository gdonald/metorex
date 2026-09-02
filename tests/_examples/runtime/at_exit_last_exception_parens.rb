at_exit do
  puts($!.class)
  puts($!.message)
end

raise("boom")
