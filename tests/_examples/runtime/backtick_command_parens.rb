target = "world"

output = `echo hello #{target}`
p(output)
p($?.exitstatus)
p($?.success?)
p($?.exited?)
p($?.stopped?)
p($?.class)

`exit 7`
p($?.exitstatus)
p($?.success?)

p(Kernel.`("echo through the module"))

class Command
  def to_str
    "echo coerced"
  end
end

p(Kernel.`(Command.new))

begin
  `nonexistent_command_xyz 2>/dev/null`
rescue Errno::ENOENT => error
  puts(error.message)
end

p(Kernel.private_instance_methods.include?(:`))
p(:`)
