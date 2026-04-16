class MSpec
  def self.protect(location, &block)
    begin
      block.call
      return true
    rescue Object => exc
      puts "outer rescue: location=#{location.inspect}, exc class=#{exc.class}"
      return false
    end
  end
end

class Matcher
  def initialize(expected_type)
    @expected_type = expected_type
  end

  def matches?(proc)
    @result = proc.call
    return false
  rescue Object => actual
    if actual.class == @expected_type
      return true
    else
      raise actual
    end
  end
end

m = Matcher.new(String)
MSpec.protect(nil) do
  m.matches?(lambda { "hello".bad_method })
end
