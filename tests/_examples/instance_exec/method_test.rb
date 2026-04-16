module MSpecMatchers
  private def equal(expected)
    "EqualMatcher:#{expected}"
  end
end

class MSpecEnv
  include MSpecMatchers
end

env = MSpecEnv.new

result = nil
env.instance_exec do
  result = equal(42)
end

puts result
