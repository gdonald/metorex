module Outer
end

$path = File.expand_path("autoload_thread_target.rb", __dir__)
Outer.autoload :Foo, $path

check = -> { Outer.autoload?(:Foo) }

before = check.call
puts "before: #{before.inspect}"

to_thread = Queue.new
from_thread = Queue.new

$saved_lambda = -> {
  v = check.call
  puts "lambda: pushing #{v.inspect}"
  from_thread.push v
  puts "lambda: pop"
  done = to_thread.pop
  puts "lambda: popped #{done.inspect}"
}

t = Thread.new {
  in_loading = from_thread.pop
  puts "thread: got #{in_loading.inspect}"
  in_other = check.call
  puts "thread: in_other = #{in_other.inspect}"
  to_thread.push :done
  [in_loading, in_other]
}

Outer.const_get(:Foo)
in_loading, in_other = t.value
after = check.call
puts "results: #{[before, in_loading, in_other, after].inspect}"
