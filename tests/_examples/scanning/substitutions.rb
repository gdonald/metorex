# `sub` puts something in place of the first match and `gsub` in place of
# every one. A replacement string can name parts of the match, a Hash looks
# one up, and a block decides as it goes.
p "hello".sub "l", "L"
p "hello".gsub "l", "L"
run_of_letters = /l+/
p "hello".gsub(run_of_letters) { |part| part.upcase }
named = /(?<letter>l)/
p "hello".gsub named, '<\k<letter>>'
pair = /(e)(l)/
p "hello".sub pair, '\2\1'
either = /[el]/
p "hello".gsub either, { "e" => "3", "l" => "1" }

# A line start is not read past the newline that closes the text.
line_start = /^/
p "Text\n".gsub line_start, " "
p "Text\nFoo".gsub line_start, " "

# A pattern on its own answers an Enumerator over the matches.
first_letter = /a/
walk = "abca".gsub first_letter
p walk.to_a
p walk.size

# The answer takes on the encoding of the first piece holding more than
# ASCII, so a replacement written in bytes decides what the answer is in.
one_letter = /l/
p "hello".gsub(one_letter) { 195.chr }.encoding

written = "hello"
written.gsub! one_letter, "y"
p written
missing = /z/
p "hello".sub! missing, "y"

# The last match the call itself made is the one left behind, whatever the
# block matched while it ran.
inner = /o/
"hello".gsub(one_letter) { "ok".match inner }
p $~[0]
p $~.string
