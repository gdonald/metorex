# Templates with Ruby in them, and running a command to see what it wrote.

require 'erb'
require 'open3'

list = %w[AAA BBB CCC]
template = ERB.new "<% list.each do |item| %><%= item %>;<% end %>"
p(template.result(binding))

# What is put into a page is escaped for it.
p(ERB::Util.html_escape "<a href='x'>&</a>")
p(ERB::Util.url_encode "a b/c")

# A template can be written onto a class as a method of its own.
Rendered = ERB.new("<%= @name %> is here").def_class Object, "render"
holder = Rendered.new
holder.instance_variable_set :@name, "metorex"
p(holder.render)

# The names a Hash carries are bound for the template to see.
p(ERB.new("<%= greeting %>, <%= subject %>").result_with_hash(greeting: "hello", subject: "world"))

# A command answers what it wrote and how it ended.
output, status = Open3.capture2 "echo written"
p(output)
p(status.exitstatus)

merged, _ = Open3.capture2e "sh -c 'echo out; echo err 1>&2'"
p(merged.split("\n").sort)

out, errors, ended = Open3.capture3 "sh -c 'echo out; echo err 1>&2'"
p([out, errors, ended.exitstatus])
