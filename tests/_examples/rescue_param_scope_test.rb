def test_method(loc, &blk)
  begin
    blk.call
    return "no error"
  rescue Object => exc
    return "caught: loc=#{loc}"
  end
end

result = test_method("my_location") { "hello".bad_method }
puts result
