# `break` returns from the call its block was attached to. Once that call
# has returned there is nothing to break out of, and `break` raises
# LocalJumpError.

def capture(&block)
  block
end

def run_it
  yield
end

captured = capture { break :done }
begin
  captured.call
rescue LocalJumpError => error
  p(error.message)
end

begin
  run_it(&captured)
rescue LocalJumpError => error
  p(error.class)
end

caught = capture do
  begin
    break :inside
  rescue LocalJumpError => error
    error.class
  end
end
p(caught.call)

worker = Thread.new do
  begin
    break :break
  rescue LocalJumpError => error
    error.class
  end
end
p(worker.value)

p(run_it { break :returned })

parent = Class.new do
  def each_value
    yield 1
  end
end
child = Class.new(parent) do
  def each_value
    super { break :from_super }
  end
end
p(child.new.each_value)

["def stops; break; end", "module Stops; break; end"].each do |source|
  begin
    eval(source)
  rescue SyntaxError
    puts("refused: #{source}")
  end
end
