# Deep stack trace example

class Deep
  def level1
    self.level2
  end

  def level2
    self.level3
  end

  def level3
    self.level4
  end

  def level4
    raise("Error at level 4!")
  end
end

begin
  obj = Deep.new
  obj.level1
rescue => e
  puts e.message
  backtrace = e.backtrace
  puts "Stack trace has #{backtrace.length} frames"
end
