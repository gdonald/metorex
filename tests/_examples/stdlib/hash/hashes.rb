# Hash/Dict methods demonstration

ages = {"alice" => 30, "bob" => 25, "charlie" => 35}

# keys and values
puts ages.keys.sort
puts ages.values.sort

# has_key?
puts ages.has_key?("alice")
puts ages.has_key?("dave")

# size
puts ages.size

# get with default
puts ages.get("alice")
puts ages.get("dave", 0)

# fetch
puts ages.fetch("bob")

# merge
merged = ages.merge({"dave" => 40})
puts merged.size

# each
total = 0
ages.each do |k, v|
  total += v
end
puts total

# entries
puts ages.entries.length
