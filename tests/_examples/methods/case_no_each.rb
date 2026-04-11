def test
  list = []
  x = 1
  case x
  when 1
    list += [x]
  end
  list
end

out = test()
puts out.length
