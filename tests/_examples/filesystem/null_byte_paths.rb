# A NUL byte ends a name the operating system reads, so a path holding one
# is refused before anything is looked up. A pattern is a string rather than
# a path, and a single glob pattern holding one is refused on its own terms.

here = Dir.pwd
[
  -> { Dir.entries(here + "\0") },
  -> { Dir.children(here + "\0") },
  -> { Dir.each_child(here + "\0").to_a },
  -> { Dir.foreach(here + "\0").to_a },
  -> { Dir.empty?(here + "\0") },
  -> { Dir.glob([here + "\0" + "*"]) },
  -> { Dir.glob(here + "\0" + "*") },
  -> { File.fnmatch("a\0", "a") },
  -> { File.fnmatch("a", "a\0") },
].each do |call|
  begin
    call.call
  rescue ArgumentError => error
    p(error.message)
  end
end
