begin
  require_relative("lib/raises_argument_error")
rescue ArgumentError => error
  puts(error.class)
  puts(error.message)
end
