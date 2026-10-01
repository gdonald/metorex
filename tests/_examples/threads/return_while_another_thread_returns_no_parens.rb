# A `return` in a block reaches its method even when another thread has
# returned from a method it entered earlier while this one waited.
$ready = false

def waiter
  loop do
    return :done if $ready
    Thread.pass
  end
end

def outer
  thread = Thread.new { waiter }
  Thread.pass
  thread
end

started = outer
$ready = true
p started.value
