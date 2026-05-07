a = "/tmp/foo/bar".split %r[/|\\]
puts a.inspect

b = "/tmp/foo/bar".split(%r[/|\\])
puts b.inspect

c = File.expand_path("/tmp/foo/bar").split %r[/|\\]
puts c.inspect

d = File.expand_path("/tmp/foo/bar").split(%r[/|\\])
puts d.inspect
