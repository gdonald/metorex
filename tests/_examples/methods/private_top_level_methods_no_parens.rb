# A method defined at the top level is a private method of Object, so every
# object answers it from inside and refuses it with a receiver, nil and true
# among them. `send` reaches it and `public_send` does not. A class's own
# methods come ahead of it, so `Dir.mkdir` is not hidden by a top-level
# `mkdir`.
require "tmpdir"

def helper = :top
def mkdir(*) = :top_level_mkdir

[Object.new, "text", 1, [1], nil, true, :name, 1.5, { a: 1 }].each do |receiver|
  p receiver.helper
rescue NoMethodError => error
  p error.message
end
p "text".send(:helper)
p [1.respond_to?(:helper), 1.respond_to?(:helper, true)]
begin
  1.public_send(:helper)
rescue NoMethodError => error
  p error.class
end
p mkdir
Dir.mktmpdir do |root|
  p Dir.mkdir(File.join(root, "made"))
  p Dir.send(:mkdir, File.join(root, "sent"))
  p Dir.exist?(File.join(root, "made"))
end
