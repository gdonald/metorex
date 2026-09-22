# Several threads asking for the same autoloaded name at once. The one that
# gets there first loads the file, the rest wait for it, and every one of them
# reads a module that is finished rather than one part-way through being built.
#
# A method that runs threads still returns from itself afterwards, which is
# what the `return` below stands for.

class Counter
  def initialize
    @value = 0
    @guard = Mutex.new
  end

  def get
    @guard.synchronize { @value }
  end

  def increment_and_get
    @guard.synchronize do
      @value += 1
      @value
    end
  end
end

class Gate
  def initialize(count)
    @count = count
    @waiting = 0
    @guard = Mutex.new
    @gate = ConditionVariable.new
  end

  def await
    @guard.synchronize do
      @waiting += 1
      if @waiting >= @count
        @waiting = 0
        @gate.broadcast
        true
      else
        @gate.wait @guard
        false
      end
    end
  end
end

$counted_loads = Counter.new

file = File.expand_path("fixtures/counted_autoload.rb", __dir__)
named = file.sub(/\.rb\Z/, "")
names = [:Counted1, :Counted2, :Counted3]
names.each { |name| Object.autoload(name, named) }

def race(names, file, hands)
  gate = Gate.new(hands)
  threads = (1..hands).map do
    Thread.new do
      names.map do |name|
        last_one_in = gate.await
        $LOADED_FEATURES.delete(file) if last_one_in && $LOADED_FEATURES.include?(file)
        gate.await
        Object.const_get(name).ready
      end
    end
  end
  return threads.map { |thread| thread.value }
end

answers = race(names, file, 4)
p answers.all? { |answered| answered == [:ready, :ready, :ready] }
p $counted_loads.get
p names.map { |name| Object.const_get(name).ready }
