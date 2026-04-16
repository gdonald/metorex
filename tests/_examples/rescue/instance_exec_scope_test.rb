class Env
  def initialize
    @name = "test_env"
  end
end

class Runner
  def initialize
    @env = Env.new
  end

  def protect(location, &block)
    begin
      @env.instance_exec(&block)
      return true
    rescue Object => exc
      puts "rescued: location=#{location}, exc=#{exc.class}"
      return false
    end
  end
end

r = Runner.new
result = r.protect("my_location") do
  "hello".bad_method
end
puts result
