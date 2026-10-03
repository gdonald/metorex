# A C extension making a Mutex, locking and unlocking it, sleeping on it,
# and running a C function while holding it.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_mutexes.c", "c_mutexes", directory))
mutexes = CMutexes.new

mutex = mutexes.make
p(mutex.class)
p(mutexes.locked(mutex))
p(mutexes.try_lock(mutex))
p(mutexes.try_lock(mutex))
report { mutexes.lock(mutex) }
p(mutexes.unlock(mutex).equal?(mutex))
report { mutexes.unlock(mutex) }
p(mutexes.lock(mutex).equal?(mutex))
p(mutexes.sleep_on(mutex, 0).class)
p(mutexes.locked(mutex))
mutex.unlock
report { mutexes.sleep_on(mutex, nil) }

p(mutexes.synchronize(mutex, -> { mutex.locked? }))
report { mutexes.synchronize(mutex, -> { raise ArgumentError, "stopped inside" }) }
p(mutex.locked?)

FileUtils.rm_rf(directory)
