child = fork
if child == nil
  exit! 7
end
p child.class
p Process.waitpid(child)
p $?.exitstatus
