# A DRb server in one process answers method calls made from another. Values
# travel by Marshal, an undumpable object stays on the server and travels as
# a reference, a block runs back in the caller, and errors come back raised.
require "drb"
require "rbconfig"

class Counter
  include DRbUndumped

  def initialize
    @count = 0
  end

  def bump(by = 1)
    @count += by
  end

  attr_reader :count
end

class Front
  def add(*numbers)
    numbers.sum
  end

  def each_doubled(list)
    list.map { |value| yield(value * 2) }
  end

  def counter
    @counter ||= Counter.new
  end

  def fail_here
    raise(ArgumentError, "bad input")
  end

  def stop
    Thread.new do
      sleep(0.1)
      DRb.stop_service
    end
    nil
  end

  private

  def hidden
    :secret
  end
end

if ARGV.first == "serve"
  DRb.start_service("druby://localhost:0", Front.new)
  STDOUT.puts(DRb.uri)
  STDOUT.flush
  DRb.thread.join
  exit
end

IO.popen([RbConfig.ruby, __FILE__, "serve"]) do |server|
  DRb.start_service("druby://localhost:0")
  remote = DRbObject.new_with_uri(server.gets.chomp)
  p(remote.add(1, 2, 3))
  p(remote.each_doubled([1, 2]) { |value| value + 1 })
  counter = remote.counter
  p(counter.class)
  counter.bump(5)
  p([counter.bump, remote.counter.count])
  begin
    remote.fail_here
  rescue ArgumentError => error
    p([error.class, error.message])
  end
  begin
    remote.hidden
  rescue NoMethodError => error
    p(error.message.sub(/ for .*/, ""))
  end
  remote.stop
  DRb.stop_service
end
