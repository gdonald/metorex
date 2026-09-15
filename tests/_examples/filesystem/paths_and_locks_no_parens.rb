# `dirname` drops the last part of a path, and a level says how many parts
# to drop.
puts File.dirname "/home/jason/poot.txt"
puts File.dirname "/home/jason/poot.txt", 2
puts File.dirname "/////foo/bar/"
puts File.dirname "poot.txt"
puts File.dirname "/foo/../."

# A lock on a whole file, taken and let go.
path = File.join Dir.tmpdir, "metorex_lock_example_no_parens"
File.write path, "held"
File.open path, "r+" do |stream|
  puts stream.flock(File::LOCK_EX).to_s
  puts stream.flock(File::LOCK_UN).to_s
end
File.delete path
