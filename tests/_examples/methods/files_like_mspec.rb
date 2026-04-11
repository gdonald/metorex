def entries(p)
  [p + "_spec.rb"]
end

def files(patterns)
  list = []
  patterns.each do |pattern|
    case pattern[0]
    when ?^
      list -= entries(pattern[1..-1])
    when ?:
      key = pattern[1..-1].to_sym
      list += files([key.to_s])
    else
      list += entries(pattern)
    end
  end
  list
end

out = files(["foo"])
puts out.length
puts out.first
