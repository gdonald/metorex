class MyClass
  def self.protect(location, &block)
    begin
      block.call
      return true
    rescue Object => exc
      return "rescued: location=#{location}"
    end
  end
end

result = MyClass.protect("test_loc") { "hello".bad_method }
puts result
