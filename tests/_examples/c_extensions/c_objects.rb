# A C extension allocating, copying, inspecting and converting objects,
# and reading and writing their instance variables, including ones C names
# without an `@`.
require "tmpdir"
require_relative "build_helper"

class Widget
  attr_reader :arguments, :block_value

  def initialize(*arguments)
    @arguments = arguments
    @block_value = yield if block_given?
  end

  def two(first, second) = [first, second]

  private

  def hidden_helper = :hidden
end

class Listing
  def to_ary = [:converted]
  def to_array = [:converted]
  def to_str = "converted"
  def to_int = 7
  def to_integer = 8
end

class Wrong
  def to_array = "not an array"
  def to_ary = :not_an_array
  def to_int = "seven"
  def to_integer = "eight"
end

class Empty
  def to_array = nil
end

class LoudEquality
  def ==(other) = :truthy
end

module Greeting
  def greet = :hello
end

class Keeper
  @@first = 1
  @@second = 2
  @own = 3
end

directory = Dir.mktmpdir
require(build_extension("c_objects.c", "c_objects", directory))
objects = CObjects.new

plain = Class.new
p(objects.allocator_kind(plain))
objects.define_allocator(plain)
p(objects.allocator_kind(plain))
made = plain.allocate
p([made.class == plain, made.instance_variable_get(:@tagged)])
Inheriting = Class.new(plain)
child = Inheriting
p(objects.allocator_kind(child))
listing = Class.new(Array)
objects.define_allocator(listing)
p([listing.allocate, listing.allocate.instance_variable_get(:@tagged)])
objects.undefine_allocator(child)
p(objects.allocator_kind(child))
report { child.allocate }
p(objects.allocator_kind(plain))

blank = objects.allocate(Widget)
p([blank.class, blank.arguments])
p(objects.call_init(blank, [1, 2]).arguments)
p(objects.call_init(objects.allocate(Widget), []) { :from_block }.block_value)
original = Widget.new(:a)
copy = objects.duplicate(original)
p([copy.arguments, copy.equal?(original)])

singled = Widget.new
singled.singleton_class
p([objects.class_of_object(singled), objects.class_of_object(5), objects.class_of_object(nil)])
p([objects.class_name(singled), objects.class_name(singled), objects.class_name(1.5)])

text = +"text"
p([objects.frozen(text), objects.frozen_by_macro(text), objects.check_frozen(text)])
p(objects.freeze(text).equal?(text))
p([objects.frozen(text), objects.frozen_by_macro(text), objects.frozen_by_macro(1)])
report { objects.check_frozen(text) }

p(objects.object_id_of(original) == original.object_id)
p([objects.instance_of(original, Widget), objects.instance_of(original, Object)])
p([objects.kind_of(original, Object), objects.kind_of(original, String)])
p(objects.method_object(original, :two).call(1, 2))
p(objects.method_object(original, "two").name)
p([objects.method_arity(original, :two), objects.method_arity(original, :arguments)])
p([objects.responds(14, :succ), objects.responds(original, :hidden_helper), objects.responds_privately(original, :hidden_helper)])
p([objects.bound(Widget, :two, true), objects.bound(Widget, :hidden_helper, true), objects.bound(Widget, :hidden_helper, false), objects.bound(Widget, :absent, false)])
p([Object.private_method_defined?(:initialize), BasicObject.private_method_defined?(:initialize), Object.private_method_defined?(:initialize, false)])

p([nil, true, false, 1, :symbol, "text", Object.new].map { |value| objects.special(value) })
p([nil, 1, Object.new].map { |value| objects.able(value) })
p([[], "", {}, Widget.new, Class.new(Array).new, Class.new(String).new, Class.new(Hash).new, STDERR, Time.now].map { |value| objects.builtin_type(value) })

p([objects.to_id("name"), objects.to_id(:name)])
report { objects.to_id(5) }

array = [1]
p(objects.check_convert(array, "to_array").equal?(array))
p(objects.check_convert(Listing.new, "to_array"))
p(objects.check_convert(Object.new, "to_array"))
p(objects.check_convert(Empty.new, "to_array"))
report { objects.check_convert(Wrong.new, "to_array") }
p(objects.convert(Listing.new, "to_array"))
report { objects.convert(Object.new, "to_array") }
report { objects.convert(nil, "to_array") }
report { objects.convert(true, "to_array") }
report { objects.convert(Empty.new, "to_array") }
subclassed = Class.new(Array).new
p(objects.check_array(subclassed).equal?(subclassed))
p([objects.check_array(Listing.new), objects.check_array(Object.new)])
report { objects.check_array(Wrong.new) }
p([objects.check_string(Listing.new), objects.check_string(5)])

p([objects.check_integer(5, "to_integer"), objects.check_integer(2**70, "to_integer")])
p([objects.check_integer(Listing.new, "to_integer"), objects.check_integer(Wrong.new, "to_integer"), objects.check_integer(Object.new, "to_integer")])
p([objects.to_int(5), objects.to_int(2.9), objects.to_int(Listing.new)])
report { objects.to_int(nil) }
report { objects.to_int("1") }
report { objects.to_int(Wrong.new) }

extended = Object.new
p(objects.extend(extended, Greeting).greet)
p(objects.instance_eval_in(original) { arguments })
p(objects.any_to_s(original).start_with?("#<Widget:0x"))
p([objects.equal(original, original), objects.equal(LoudEquality.new, 1), objects.equal(1, 2)])
p([objects.inherited(Array, Array), objects.inherited(Array, Object), objects.inherited(Array, Hash)])
report { objects.inherited(1, 2) }
report { objects.inherited(Array, 2) }
p([objects.require_feature(File.join(__dir__, "required_by_c")), $required_by_c])

p([objects.respond_to?(:unavailable), objects.respond_to?(:unavailable, true)])
report { objects.unavailable }

holder = Widget.new
p(objects.ivar_set(holder, :@visible, 1))
p([objects.ivar_get(holder, :@visible), objects.ivar_defined(holder, :@visible), objects.ivar_defined(holder, :@absent)])
p(objects.ivar_set(holder, :hidden, 2))
p([objects.ivar_get(holder, :hidden), objects.ivar_defined(holder, :hidden), objects.ivar_get(holder, :other), objects.ivar_defined(holder, :other)])
p(objects.iv_set(holder, "bare", 3))
p([objects.iv_get(holder, "bare"), objects.iv_get(holder, "@visible"), objects.attr_get(holder, :@visible)])
p([holder.instance_variables, objects.instance_variables_of(holder)])
p(objects.ivar_count(holder))
p(objects.ivar_foreach(holder))
p(objects.ivar_foreach(Keeper))
p(objects.ivar_foreach(Widget.new))
source = +"source"
source.instance_variable_set(:@note, :kept)
target = objects.copy_ivars(+"target", source)
p(target.instance_variable_get(:@note))
objects.free_ivars(holder)
p([holder.instance_variables, objects.ivar_get(holder, :hidden)])

FileUtils.rm_rf(directory)
