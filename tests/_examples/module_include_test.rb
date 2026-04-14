module MSpecMatchers
  private def equal(expected)
    "matcher:#{expected}"
  end
end

class MSpecEnv
  include MSpecMatchers
end

env = MSpecEnv.new
result = env.equal("hello")
puts result
