def test
  list = []
  x = 1
  case x
  when 1
    list += [x]
  end
  puts "after case list=#{list.length}"
  return list
end

out = test()
puts out.length
