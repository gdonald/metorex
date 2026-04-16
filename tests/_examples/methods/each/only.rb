def test
  list = []
  [1].each do |x|
    list += [x]
  end
  list
end

out = test()
puts out.length
