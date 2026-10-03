# A C extension built with mkmf and make, loaded with require, and the
# methods it defines called from Ruby.
require "tmpdir"
require_relative "build_helper"

class Base
  def self.inherited(subclass)
    puts "inherited by #{subclass}"
    super
  end
end

directory = Dir.mktmpdir
p(require(build_extension("c_methods.c", "c_methods", directory)))

methods = CMethods.new
p(CMethods.superclass)
p((0..15).map { |arity| methods.public_send("arity_#{arity}", *(1..arity).to_a) })
p(methods.counted(:a, :b, :c))
p(methods.gathered(1, "two", :three))
p(methods.same(:symbol, :symbol))
text = "text"
p(methods.same(text, text))
p(methods.same("text", "text"))
held_by_reference = [{ a: 1 }, method(:puts), -> {}, StandardError.new("failed"), Set[1], binding, /x/, 1..2]
p(held_by_reference.map { |held| methods.same(held, methods.arity_1(held)) })
p(methods.arity_1(2**70))
p(methods.arity_1(1.5))
p([methods.class_of(nil), methods.class_of(true), methods.class_of(false), methods.class_of(3)])
object = Object.new
p(methods.class_of(object) == Object)
singleton = object.singleton_class
p(methods.class_of(object) == singleton)
p(methods.class_of(CMethods) == CMethods.singleton_class)
p(methods.unknown_handle)
p(methods.reopen.equal?(CMethods))
p(methods.child_of(Base))
report { methods.arity_2(1) }
report { methods.mismatch }
report { methods.integer_superclass }
report { methods.constant_taken }
report { methods.arity_too_large }
report { methods.method_on_integer }

no_init_directory = Dir.mktmpdir
begin
  require(build_extension("no_init.c", "no_init", no_init_directory))
rescue LoadError => error
  p(error.class)
end
p($LOADED_FEATURES.any? { |feature| feature.include?("no_init") })

not_a_library = File.join(no_init_directory, "plain_text.#{RbConfig::CONFIG["DLEXT"]}")
File.write(not_a_library, "plain text")
begin
  require(not_a_library)
rescue LoadError => error
  p(error.class)
end

FileUtils.rm_rf(directory)
FileUtils.rm_rf(no_init_directory)
