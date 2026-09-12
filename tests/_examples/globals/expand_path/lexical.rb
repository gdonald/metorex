# expand_path reads the path it was given, never the filesystem, so a
# symlink along the way stands as it was written.
puts(File.expand_path("/usr/../tmp"))
puts(File.expand_path("..", "/one/two/three"))
puts(File.expand_path("/.."))
puts(File.expand_path("./four", "/one/two"))
