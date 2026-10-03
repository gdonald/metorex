# A C extension working with classes and modules: constants under any name,
# aliases, methods of each visibility and arity, singleton and module
# functions, undefining methods, names and ancestors. An extension can also
# be autoloaded and required by a path written without its ending.
require "tmpdir"
require "stringio"
require_relative "build_helper"

class Base
  INHERITED = :from_base
end

class Derived < Base
  OWN = :own
  autoload :LATER, File.join(__dir__, "autoloaded_class.rb")

  def self.const_missing name
    "missing #{name}"
  end
end

module Plain
end

directory = Dir.mktmpdir
require build_extension("c_modules.c", "c_modules", directory)
modules = CModules.new

p [modules.defined(Derived, :OWN), modules.defined(Derived, :INHERITED), modules.defined(Plain, :String)]
p [modules.get(Derived, :INHERITED), modules.get(Plain, :String)]
p [modules.get_at(Derived, :OWN), modules.get_at(Derived, :INHERITED)]
p [modules.get_from(Derived, :INHERITED), modules.get_from(Derived, :String)]
modules.set Derived, :_hidden, 7
p [modules.get(Derived, :_hidden), modules.defined(Derived, :_hidden)]
report { modules.get Base, :_hidden }
report { Derived.const_get :_hidden }
$stderr = StringIO.new
modules.set Derived, :OWN, :replaced
warned = $stderr.string
$stderr = STDERR
p [Derived::OWN, warned.include?("already initialized constant Derived::OWN")]
modules.global_const "CModulesGlobal", 42
p CModulesGlobal
FrozenPlain = Module.new.freeze
report { modules.set FrozenPlain, :Frozen, 1 }

class Aliased
  def original
    :original
  end
end
modules.aliases Aliased
p [Aliased.new.first_alias, Aliased.new.second_alias]

class Holder
end
holder = Holder
modules.methods_on holder
made = holder.new
p [made.plain, made.one(1), made.any(1, 2, 3), made.listed(1, 2)]
p(%i[plain one any listed].map { |name| holder.instance_method(name).arity })
p [holder.private_instance_methods(false).include?(:hidden), holder.protected_instance_methods(false).include?(:guarded)]
report { made.hidden }
p made.send(:hidden)

single = Object.new
modules.singleton single
p single.only_here
report { Object.new.only_here }

functions = Module.new
modules.module_function functions
p [functions.both_ways, functions.method(:both_ways).arity, functions.private_instance_methods.include?(:both_ways)]
modules.global_function
p c_modules_everywhere

class Undone
  def removed
    :removed
  end
end
modules.undefine Undone, "removed"
modules.undefine Undone, "never_defined"
report { Undone.new.removed }
p Undone.instance_methods(false)
modules.undefine Undone, "initialize_copy"
report { Undone.new.dup }
FrozenUndone = Class.new.freeze
report { modules.undefine FrozenUndone, "anything" }
report { modules.undefine_strictly Undone, :not_there }

p modules.naming(Derived)
p modules.ancestors(Module.new.tap { |held| held.include Comparable }).last

module AutoloadHolder
  autoload :FromExtension, File.join(Dir.mktmpdir, "c_autoloaded")
end
autoload_directory = File.dirname AutoloadHolder.autoload?(:FromExtension)
built = build_extension "c_autoloaded.c", "c_autoloaded", autoload_directory
p AutoloadHolder::FromExtension.name
p require(built.delete_suffix(File.extname(built)))

FileUtils.rm_rf directory
FileUtils.rm_rf autoload_directory
