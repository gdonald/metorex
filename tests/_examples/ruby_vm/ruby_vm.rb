# RubyVM names MRI's instructions and settings, answers MRI's counters, and
# remembers whether script lines are kept. It cannot be instantiated, and
# neither just-in-time compiler is running.

p(RubyVM.class, RubyVM.superclass)
p(RubyVM::OPTS)
p(RubyVM::INSTRUCTION_NAMES.size, RubyVM::INSTRUCTION_NAMES.first(5), RubyVM::INSTRUCTION_NAMES.frozen?)
p(RubyVM::DEFAULT_PARAMS)
p(RubyVM.stat.keys)
p(RubyVM.stat(:constant_cache_invalidations).is_a?(Integer))
held = {}
p(RubyVM.stat(held).equal?(held))
p(RubyVM.keep_script_lines)
RubyVM.keep_script_lines = true
p(RubyVM.keep_script_lines)
p(RubyVM.singleton_methods.sort)
def attempt
  yield
rescue => e
  [e.class, e.message]
end
p(attempt { RubyVM.stat(:nope) })
p(attempt { RubyVM.stat("x") })
p(attempt { RubyVM.new })
p(attempt { RubyVM.allocate })
p(defined?(RubyVM::YJIT), defined?(RubyVM::ZJIT))
p(RubyVM::YJIT.enabled?, RubyVM::ZJIT.enabled?)
