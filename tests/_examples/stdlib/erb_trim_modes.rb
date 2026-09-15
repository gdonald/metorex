# An ERB trim mode decides what a tag leaves behind: `>` drops the newline
# after a tag that ends a line, `<>` only when the tag opens the line too,
# `-` trims where `<%-` and `-%>` say to, and `%` reads a whole line of code.
require "erb"

template = "<ul>\n<% [1, 2].each do |item| %>\n<li><%= item %></li>\n<% end %>\n</ul>\n"
p ERB.new(template).result
p ERB.new(template, trim_mode: ">").result
p ERB.new(template, trim_mode: "<>").result

explicit = "<ul>\n<%- [1, 2].each do |item| -%>\n<li><%= item %></li>\n<%- end -%>\n</ul>\n"
p ERB.new(explicit, trim_mode: "-").result

percent = "<ul>\n% [1, 2].each do |item|\n<li><%= item %></li>\n% end\n</ul>\n%%done\n"
p ERB.new(percent, trim_mode: "%").result
