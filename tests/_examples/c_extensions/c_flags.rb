# A C extension reading and writing the flags and class of objects through
# RBASIC, freezing and thawing them with the FREEZE flag.
require "tmpdir"
require_relative "build_helper"

class Widget; end

directory = Dir.mktmpdir
require(build_extension("c_flags.c", "c_flags", directory))
flags = CFlags.new
freeze = flags.freeze_flag
user = flags.user_flag

p([freeze, user])
widget = Widget.new
p(flags.flags(widget))
widget.freeze
p(flags.flags(widget) == freeze)
flags.set_flags(widget, 0)
p(widget.frozen?)
flags.set_flags(widget, freeze)
p(widget.frozen?)

text = +"text"
flags.set_flags(text, freeze)
p(text.frozen?)
flags.set_flags(text, 0)
p(text.frozen?)
list = [1]
flags.set_flags(list, freeze)
p(list.frozen?)
flags.set_flags(list, 0)
list << 2
p(list)
named = Class.new
flags.set_flags(named, freeze)
p(named.frozen?)
flags.set_flags(named, 0)
p(named.frozen?)
p(flags.set_flags(1.5, 0).frozen?)

kept = Widget.new
flags.set_user_flag(kept)
p(flags.flags(kept) == user)
p(kept.frozen?)
flags.unset_user_flag(kept)
p(flags.flags(kept))

p(flags.klass(Widget.new))
plain = Widget.new
plain.singleton_class
p(flags.klass(plain) == plain.singleton_class)

p(flags.test_flag(1))
p(flags.test_flag(nil))
p(flags.test_flag(:symbol) == freeze)
p([flags.special_const(1), flags.special_const(nil), flags.special_const(false), flags.special_const(Widget.new)])

FileUtils.rm_rf(directory)
