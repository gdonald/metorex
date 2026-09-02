at_exit do
  puts("in at_exit")
  puts("$! is #{$!.class}:#{$!.message}")
  exit!(21)
end

raise("original error")
