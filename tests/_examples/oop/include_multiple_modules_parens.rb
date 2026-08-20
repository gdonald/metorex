module First
  def first_name
    :first
  end
end

module Second
  def second_name
    :second
  end
end

module Wrapper
  module Third
    def third_name
      :third
    end
  end
end

class Host
  include First, Second, Wrapper::Third
end

host = Host.new
puts host.first_name.inspect
puts host.second_name.inspect
puts host.third_name.inspect
puts Host.ancestors[0, 4].inspect
puts Host.include?(Second).inspect
puts First.include?(First).inspect

begin
  Host.include(Class.new)
rescue TypeError => error
  puts error.class
end

begin
  Class.new do
    include
  end
rescue ArgumentError => error
  puts error.class
end
