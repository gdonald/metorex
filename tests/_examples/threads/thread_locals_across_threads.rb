# One thread reads what another has put in its own store, which is what a
# thread waiting on another's progress watches.
held = Thread.new do
  Thread.current[:marker] = true
  sleep 0.1
  :done
end

watcher = Thread.new do
  Thread.pass until held[:marker]
  :saw_it
end

puts watcher.value.inspect
puts held.value.inspect
puts held[:marker].inspect
