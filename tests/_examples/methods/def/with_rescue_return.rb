def parse_args(argv)
  result = []
  while entry = argv.shift
    result << entry
  end
  result
rescue => e
  puts "rescued: #{e}"
  []
end

out = parse_args(["a", "b", "c"])
puts out.inspect
puts out.empty?
