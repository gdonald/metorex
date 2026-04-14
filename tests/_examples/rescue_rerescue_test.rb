class MSpec
  def self.protect(location, &block)
    begin
      block.call
      return true
    rescue Object => exc
      puts "rescue caught: location=#{location.inspect}"
      return false
    end
  end
end

class Matcher
  def matches?(proc)
    begin
      @result = proc.call
      return false
    rescue Object => actual
      # Exception was wrong type, re-raise
      raise actual
    end
  end
end

m = Matcher.new
MSpec.protect("test_loc") do
  m.matches?(lambda { "hello".bad_method })
end
