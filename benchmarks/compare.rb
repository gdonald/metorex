# Times each workload under MRI and under metorex and prints a table.
# Run by hand from the repository root with a release build:
#
#   cargo build --release
#   ruby benchmarks/compare.rb
#
# Another metorex binary can be named as the first argument.

require "open3"

metorex = ARGV.fetch(0, File.expand_path("../target/release/metorex", __dir__))
abort "no metorex binary at #{metorex}" unless File.executable?(metorex)

# One run of `program` on `script`, answering the seconds it took and what it
# printed.
def timed(program, script)
  started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  printed, status = Open3.capture2e(program, script)
  finished = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  abort "#{program} #{script} failed:\n#{printed}" unless status.success?
  [finished - started, printed]
end

# The fastest of three runs, which leaves out a run slowed by something else
# on the machine.
def fastest(program, script)
  runs = Array.new(3) { timed(program, script) }
  outputs = runs.map(&:last).uniq
  abort "#{program} #{script} printed different things" unless outputs.size == 1
  [runs.map(&:first).min, outputs.first]
end

puts format("%-20s %10s %10s %8s", "workload", "MRI", "metorex", "ratio")
Dir[File.join(__dir__, "workloads", "*.rb")].sort.each do |script|
  mri_seconds, mri_output = fastest(RbConfig.ruby, script)
  metorex_seconds, metorex_output = fastest(metorex, script)
  abort "#{script} printed different things under MRI and metorex" unless mri_output == metorex_output
  puts format("%-20s %9.3fs %9.3fs %7.1fx", File.basename(script, ".rb"), mri_seconds,
              metorex_seconds, metorex_seconds / mri_seconds)
end
