def parse_args(argv)
  result = []
  while entry = argv.shift
    result << entry
  end
  result
end

out = parse_args(["a", "b", "c"])
puts out.length
