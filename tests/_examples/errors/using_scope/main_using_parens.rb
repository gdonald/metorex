MAIN = self

module X
  begin
    MAIN.send(:using, Module.new)
  rescue RuntimeError => error
    puts(error.message)
  end
end

MAIN.send(:using, Module.new)
puts("toplevel using allowed")
