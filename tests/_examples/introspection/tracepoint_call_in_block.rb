# A method defined and called inside a block reports a :call event naming the
# method, the file, and the line it was defined on, however the block is run.

def run_it
  yield
end

class Runner
  def run(&block)
    Object.new.instance_exec(&block)
  end
end

seen = []
run_it do
  TracePoint.new(:call) { |trace| seen << [trace.method_id, trace.lineno] }.enable do
    def defined_in_block; end
    defined_in_block
  end
end
p(seen)

inspected = []
Runner.new.run do
  TracePoint.new(:call) { |trace|
    next unless trace.path == __FILE__
    inspected << trace.inspect.sub(__FILE__, "FILE")
  }.enable do
    def defined_in_instance_exec; end
    defined_in_instance_exec
  end
end
p(inspected)

paths = []
Runner.new.run do
  TracePoint.new(:call) { |trace| paths << [trace.method_id, trace.path == __FILE__] }.enable do
    def defined_again; end
    defined_again
  end
end
p(paths)
p(Runner.private_instance_methods(false).include?(:defined_again))
