# `\z` stands at the very end of the subject, and `\Z` stands there too or
# just before a newline that ends it.
p "abc" =~ /c\z/
p "abc\n" =~ /c\z/
p "abc" =~ /c\Z/
p "abc\n" =~ /c\Z/
p "file.rb".sub(/\.rb\Z/, "")
p "undefined method 'foo' for nil" =~ /\Aundefined method 'foo' for nil\Z/
