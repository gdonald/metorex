def files(patterns)
  list = []
  patterns.each do |pattern|
    list += [pattern]
  end
  list
end

out = files(["a", "b"])
puts out.length
