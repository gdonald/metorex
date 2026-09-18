# A paired delimiter nests, so a group written inside one does not close the
# literal, and an unescaped slash is written back escaped.
puts(%r( () [c]{1} ).source)
puts(%r[/].to_s)

# A letter, a space, or an unclosed delimiter opens no literal at all.
["%ra foo a", "%r !foo!", "%r{ foo {"].each do |source|
  begin
    eval(source)
    puts("no error for #{source}")
  rescue SyntaxError
    puts("SyntaxError")
  end
end

# A call to a named group reads its pattern again, and the group takes what
# that read.
named_call = /(?<foo>foo.)bar\g<foo>/
puts(named_call.match("foo1barfoo2").to_a.inspect)

# A pattern written on its own as a condition matches against the last line
# read, and says so.
$_ = "hay"
puts((true if /hay/).inspect)
$_ = "straw"
puts((true if /hay/).inspect)
