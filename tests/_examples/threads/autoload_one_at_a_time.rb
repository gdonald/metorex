# A constant is loaded once however many threads ask for it. The one that
# arrives second waits for the first rather than loading it again.
module Holder
  def self.recorded
    @recorded ||= []
  end
end

Holder.autoload :Loaded, File.expand_path("fixtures/slow_autoload.rb", __dir__)

start = false
first_value = nil
second_value = nil

first = Thread.new do
  Thread.pass until start
  first_value = Holder::Loaded
  Holder.recorded << :first_done
end

second = Thread.new do
  Thread.pass until first && first[:loading]
  second_value = Holder::Loaded
  Holder.recorded << :second_done
end

start = true
first.join
second.join

puts Holder.recorded.inspect
puts [first_value, second_value].inspect
