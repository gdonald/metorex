main_id = Thread.main.native_thread_id
p main_id.class
p main_id.positive?
p main_id == Thread.main.native_thread_id
waiting = Thread.new { sleep }
Thread.pass until waiting.status == "sleep"
p waiting.native_thread_id != main_id
waiting.kill
waiting.join
p waiting.native_thread_id
finished = Thread.new { Thread.current.native_thread_id.class }
p finished.value
p finished.native_thread_id
fresh = Thread.new { }
p fresh.native_thread_id != main_id
fresh.join

# The settings for running threads M:N are accepted.
require "rbconfig"
settings = { "RUBY_MN_THREADS" => "1", "RUBY_MAX_CPU" => "2" }
system settings, RbConfig.ruby, "-e", "p Thread.new { :ran }.value"
p $?.success?
