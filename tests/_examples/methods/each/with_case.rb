def test
  list = []
  [1].each do |x|
    case x
    when 1
      list += [x]
    end
  end
  list
end

out = test()
puts out.length
