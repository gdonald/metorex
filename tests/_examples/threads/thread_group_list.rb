# A ThreadGroup lists the living threads in it: the main thread and the
# threads it starts belong to the default group until another group takes
# one, and a thread that has finished is listed in none.
p(Thread.main.group.equal?(ThreadGroup::Default))
p(ThreadGroup::Default.list.include?(Thread.main))

ready = Queue.new
release = Queue.new
worker = Thread.new do
  ready << :started
  release.pop
end
ready.pop
p(ThreadGroup::Default.list.include?(worker))

own = ThreadGroup.new
p(own.list)
own.add(worker)
p([own.list.include?(worker), ThreadGroup::Default.list.include?(worker), worker.group.equal?(own)])

release << :go
worker.join
p([own.list, ThreadGroup::Default.list.include?(worker)])
