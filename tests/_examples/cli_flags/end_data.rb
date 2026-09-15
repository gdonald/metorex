# A line reading `__END__` closes the code, and `DATA` reads the text after it
# as a file positioned where that text begins.
p DATA.class
p DATA.pos > 0
p DATA.read
DATA.rewind
p DATA.gets
__END__
first line
second line
