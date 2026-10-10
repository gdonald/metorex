=begin
Lines between =begin and =end at the start of a line are a comment.
p "never printed"
=end
p "after the first document"
=begin with words after the marker
=end and words after this one too
rate = 7
=begin
  =end indented does not close it
p "still a comment"
=end
p rate
