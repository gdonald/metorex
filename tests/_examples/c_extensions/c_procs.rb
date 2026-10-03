# A C extension making a Proc around a C block function, asking a Proc its
# arity, and calling Procs with arguments, keywords and a block.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_procs.c", "c_procs", directory))
procs = CProcs.new

made = procs.make(:data)
p(made.call(1, 2))
p(made.call([1, 2]))
p(made.call { :given })
p(made[])
p([made.arity, made.lambda?, made.source_location])
p(made.inspect.match?(/\A#<Proc:0x\h+>\z/))
p(made == procs.make(:data))
reader = procs.make_block_reader
action = -> { :acted }
p(reader.call(&action).equal?(action))
p(reader.call)

p([procs.arity(proc { |a, b| }), procs.arity(->(a, *b) {}), procs.arity(made)])
p([procs.is_proc(made), procs.is_proc(-> {}), procs.is_proc(Class.new(Proc).new {}), procs.is_proc(:sym), procs.is_proc(nil)])
p(procs.call(proc { |a, b| a * b }, [6, 7]))
report { procs.call(proc {}, 5) }
taking = proc { |*rest, **keywords| [rest, keywords] }
p(procs.call_keywords(taking, [{}]))
p(procs.call_keywords(taking, [{ b: 2 }, { a: 1 }]))
p(procs.call_keywords(taking, []))
report { procs.call_keywords(taking, [1, 2]) }
doubling = proc { |a, b, &block| block ? block.call(a * b) : a * b }
p(procs.call_block(doubling, [6, 7], proc { |n| n * 2 }))
p(procs.call_block(doubling, [6, 7], nil))
with_block = proc { |*rest, **keywords, &block| [rest, keywords, block&.call] }
p(procs.call_block_keywords(with_block, [{ a: 1 }], proc { 42 }))
p(procs.call_block_keywords(with_block, [{}], nil))

FileUtils.rm_rf(directory)
