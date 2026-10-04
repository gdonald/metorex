# A C extension running Ruby code that may raise, throw or break through
# rb_protect, rb_rescue, rb_rescue2 and rb_ensure, and calling the other
# Kernel functions C reaches for.
require "stringio"
require "tmpdir"
require_relative "build_helper"

class Hidden
  def shown = :shown

  private

  def concealed = :concealed
end

class Responder
  def respond_to?(name, include_private = false)
    name == :ghost || super
  end

  def method_missing(name, *arguments)
    name == :ghost ? :answered_by_method_missing : super
  end
end

def keywords(*positional, **named, &block)
  [positional, named, block&.call]
end

directory = Dir.mktmpdir
require(build_extension("c_kernel.c", "c_kernel", directory))
kernel = CKernel.new

p(kernel.protect(1) { |value| value + 1 })
answered, state, info = kernel.protect(1) { raise ArgumentError, "inside protect" }
p([answered, state, info.class, info.message])
kernel.clear_errinfo
p(kernel.protect(1) { |value| break value * 10 })
p(catch(:done) { kernel.protect(1) { throw :done, :thrown } })
p(kernel.protect_then_jump(2) { |value| value * 3 })
report { kernel.protect_then_jump(2) { raise KeyError, "jumped" } }
p(kernel.protect_then_jump(2) { |value| break value * 4 })
report { kernel.jump_without_protect }

body = ->(value) { raise ArgumentError, "body #{value}" }
handler = ->(value, exception) { [value, exception.class, $!.equal?(exception)] }
p(kernel.rescue([->(value) { value * 2 }, 21], [handler, :unused]))
p(kernel.rescue([body, 1], [handler, :handled]))
p(kernel.rescue([body, 1], nil))
begin
  kernel.rescue([->(_) { raise Exception, "not standard" }, 1], [handler, :unused])
rescue Exception => error
  p([error.class, error.message])
end
report { kernel.rescue([body, 1], [->(*) { raise IOError, "from handler" }, :unused]) }
p($!)
p(kernel.rescue2([body, 2], [handler, :matched], TypeError, ArgumentError))
report { kernel.rescue2([body, 2], [handler, :unmatched], TypeError, KeyError) }
report { kernel.rescue2([body, 2], [handler, :unused], Object.new, 42) }
p(kernel.rescue2([->(value) { value }, :no_error], [handler, :unused], TypeError, KeyError))

p(kernel.rescue([->(_) { nil.undefined_thing }, 1], [->(_, exception) { exception.class }, nil]))
p(kernel.rescue([->(_) { 1 + "a" }, 1], [->(_, exception) { exception.class }, nil]))
p(kernel.protect(1) { 1 + "a" }[1])
p(kernel.rescue([->(_) { raise 5 }, 1], [->(_, exception) { exception.class }, nil]))
kernel.clear_errinfo

ensured = []
p(kernel.ensure([->(value) { value }, :body], [->(value) { ensured << [value, $!] }, :clean]))
report { kernel.ensure([body, 3], [->(value) { ensured << [value, $!.class] }, :after_raise]) }
p(ensured)
p(kernel.ensure([->(value) { [1, 2].each { |item| break item * value } }, 5], [->(_) { ensured << :after_break }, nil]))

p(kernel.catch_named("stop", ->(tag) { throw tag, [tag, :caught] }))
p(kernel.catch_named("stop", ->(tag) { :no_throw }))
tag = Object.new
p(kernel.catch_object(tag, ->(given) { throw given, given.equal?(tag) }))
p(catch(:named) { kernel.throw_named("named", :from_c) })
p(catch(tag) { kernel.throw_object(tag, :object_from_c) })
report { kernel.throw_named("nobody", 1) }

local = 40
p(kernel.evaluate("local + 2"))
p(kernel.evaluate_protected("local + 3"))
p(kernel.evaluate_protected("raise 'evaluated'"))

p(kernel.recurse(:object, 0))
p(kernel.recurse(:object, 2))
report { kernel.recurse_raising("raised while recursing") }
report { kernel.recurse_raising("raised again, so the first call was let go") }

report { kernel.need_block }
p(kernel.need_block { :given })
made = kernel.block_lambda { :literal }
p([made.lambda?, made.call])
existing = proc { :existing }
p(kernel.block_lambda(&existing).equal?(existing))
already = lambda { :already }
p(kernel.block_lambda(&already).equal?(already))
p(kernel.this_func)
p(CKernel::FRAME_AT_LOAD)

report { kernel.sys_fail("while opening") }
report { kernel.sys_fail(nil) }
report { kernel.syserr_fail(Errno::EINVAL::Errno, "bad value") }
report { kernel.syserr_fail(Errno::EINVAL::Errno, nil) }
report { kernel.syserr_fail_str(Errno::EACCES::Errno, "no access") }

p(kernel.keyword_given(a: 1))
p(kernel.keyword_given({ a: 1 }))
p(kernel.call_with_keywords(kernel, :keyword_given, [{ a: 1 }]))
p(kernel.call_with_keywords(kernel, :keyword_given, [{}]))
p(kernel.call_with_keywords(self, :keywords, [1, {}]))
p(kernel.call_without_keywords(kernel, :keyword_given, [{ a: 1 }]))
p(kernel.call_with_keywords(self, :keywords, [1, { b: 2 }]))
report { kernel.call_with_keywords(self, :keywords, [1, 2]) }
p(kernel.call_public(Hidden.new, :shown))
report { kernel.call_public(Hidden.new, :concealed) }
p(kernel.call_with_block(self, :keywords, [1], proc { :from_block }))
p(kernel.call_with_block(self, :keywords, [1], nil))
p(kernel.call_with_block_and_keywords(self, :keywords, [1, { c: 3 }], proc { :both }))
report { kernel.call_with_block(Hidden.new, :concealed, [], proc {}) }
p(kernel.check_call(Hidden.new, :shown))
p(kernel.check_call(Hidden.new, :absent))
p(kernel.check_call(Responder.new, :ghost))

p(kernel.sprintf_values(["%d %f %s", 10, 2.5, "text"]))
p(kernel.format_values("%05.1f|%-4s|", [3.14159, "ab"]))
p(kernel.backtrace.first.include?(__FILE__))

$stderr = StringIO.new
Warning[:deprecated] = true
Warning[:experimental] = false
Warning[:performance] = true
kernel.category_warn(0, "uncategorized")
kernel.category_warn(1, "deprecated")
kernel.category_warn(2, "experimental is off")
kernel.category_warn(3, "performance")
$VERBOSE = nil
kernel.category_warn(1, "silenced")
$VERBOSE = false
warnings = $stderr.string
$stderr = STDERR
puts(warnings.gsub(/^.*:(\d+): /, "line \\1: "))

p(kernel.map_with_own_block([1, 2]) { |item| item + 1 })
p(kernel.map_with_own_block([1, 2]).class)

kernel.end_proc($stdout)
kernel.vm_exit_hook
at_exit { puts "ruby at_exit ran" }
FileUtils.rm_rf(directory)
puts "end of script"
