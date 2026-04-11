def files(patterns)
  list = []
  patterns.each do |pattern|
    case pattern
    when "x"
      list += ["X"]
    else
      list += [pattern]
    end
  end
  list
end

out = files(["foo"])
puts out.length
