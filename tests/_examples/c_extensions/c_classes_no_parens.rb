# A C extension working with classes: calling the method a C method
# overrides, paths, instance methods by visibility, new classes and
# instances, superclasses, class variables, attributes, included modules,
# type tags, and the checks rb_define_class_under makes.
require "tmpdir"
require_relative "build_helper"

class Greeting
  def text
    "hello"
  end

  def repeat count
    "hello " * count
  end

  def itself_from_super
    self
  end
end

class LoudGreeting < Greeting
end

class LouderGreeting < LoudGreeting
end

directory = Dir.mktmpdir
require build_extension("c_classes.c", "c_classes", directory)
classes = CClasses.new

classes.super_calling LoudGreeting, "text"
classes.super_calling LouderGreeting, "text"
classes.super_calling_with LoudGreeting, "repeat"
classes.super_calling LoudGreeting, "itself_from_super"
classes.super_calling LoudGreeting, "nothing_above"
loud = LoudGreeting.new
p [LouderGreeting.new.text, loud.repeat(2), loud.itself_from_super.equal?(loud)]
report { loud.nothing_above }

module Outer
  class Inner
  end

  VALUE = 1
end
p [classes.path(Outer::Inner), classes.path(File::Stat)]
p [classes.path2class("Outer::Inner"), classes.path_to_class("Outer")]
report { classes.path2class "Outer::Missing" }
report { classes.path2class "Outer::String" }
report { classes.path_to_class "Outer::VALUE" }

class Visibilities
  def shown
  end

  protected

  def guarded
  end

  private

  def hidden
  end
end
p classes.instance_methods_of(Visibilities, false)

made = classes.class_new(Greeting)
p [made.superclass, made.name, made.new.text]
report { classes.class_new Class }
report { classes.class_new Object.new.singleton_class }
report { classes.class_new Comparable }

class Options
  attr_reader :positional, :keywords

  def initialize(*positional, **keywords)
    @positional = positional
    @keywords = keywords
  end
end
built = classes.new_instance_kw([{ first: 1 }, { second: 2 }], Options)
p [built.positional, built.keywords]
report { classes.new_instance_kw [42], Options }

special = LoudGreeting.new
def special.extra
end
p [classes.real(special), classes.real(LoudGreeting.new), classes.real_of_null]
p [classes.superclasses(LoudGreeting), classes.superclasses(BasicObject)]
report { classes.superclasses Comparable }

class Counted
  @@count = 0
  @level = 1
end
p [classes.cvar_defined(Counted, "@@count"), classes.cvar_defined(Counted, "@@other"), classes.cvar_defined(Counted, "@level")]
p classes.cvars(Counted)
p Counted.class_variables.sort
report { classes.cv_get Counted, "@@missing" }

class Stored
  def initialize
    @readable = 1
    @both = 3
  end
end
classes.attrs Stored
stored = Stored.new
stored.writable = 2
stored.both = 4
p [stored.readable, stored.instance_variable_get(:@writable), stored.both]
report { stored.readable = 5 }

module Waves
  def wave
    :waving
  end
end
p classes.include_module(Class.new, Waves).new.wave

p [1, nil, true, false, 1.5, "text", :text, [], {}, Object, Kernel, 2**70, 1..2, Object.new].map { |held| classes.type_of(held) }
p [classes.is_type(1, CClasses::T_FIXNUM), classes.is_type("text", CClasses::T_STRING), classes.is_type(1, CClasses::T_STRING)]

report { classes.define_class Outer, "NoParent", nil }
report { classes.define_class Outer, "FromModule", Comparable }
report { classes.define_class Outer, "Inner", Greeting }
report { classes.define_class Outer, "VALUE", Object }
report { classes.define_class Object, "LoudGreeting", Object }
hidden = classes.define_class(Outer, "_hidden", Greeting)
p [hidden.name, classes.define_class(Outer, "_hidden", Greeting).equal?(hidden)]

FileUtils.rm_rf directory
