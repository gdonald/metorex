# `each_line` and `lines` cut a string on a separator. The separator defaults
# to what `$/` names, an empty one cuts the text into paragraphs, and `nil`
# leaves the whole string in one piece.
p "one\ntwo\nthree".lines
p "one\ntwo\nthree".lines nil
p "hello\nworld\n\n\nand\nuniverse\n\n\n\n\n".lines ""
p "hello world".lines " "

# `chomp:` takes the separator back off each piece, and a walk with no
# separator of its own takes a carriage return off with the newline.
p "hello \r\nworld\r\n".lines chomp: true
p "hello world".lines " ", chomp: true

# A separator is read through `to_str`.
class Divider
  def to_str
    "l"
  end
end
p "hello\nworld".lines Divider.new

collected = []
"one\ntwo".each_line { |line| collected << line }
p collected

# `$/` names the separator a walk with none of its own cuts on.
$/ = "x"
p "axbxc".lines
$/ = "\n"
