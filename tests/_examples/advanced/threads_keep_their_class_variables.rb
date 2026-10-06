class Worker
  @@label = "worker"

  def work
    $in_work = true
    until $stop
    end
    @@label
  end
end

class Reader
  @@label = "reader"

  def read
    Thread.pass until $in_work
    label = @@label
    $stop = true
    label
  end
end

worker = Thread.new { Worker.new.work }
p(Reader.new.read)
p(worker.value)
