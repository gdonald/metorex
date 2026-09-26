require("open3")

# `popen3` joins the command's three streams to pipes, and its thread names
# the command and answers the status it ended with.
Open3.popen3("cat; echo err 1>&2") do |input, output, errors, waiter|
  input.write("to the command")
  input.close()
  p(output.read())
  p(errors.read())
  p(waiter.pid().is_a?(Integer))
  p(waiter.value().success?())
end

# The capturing forms hand the command what it is to read.
p(Open3.capture2("tr a-z A-Z", stdin_data: "shout")[0])
out, err, status = Open3.capture3("cat; exit 4", stdin_data: "kept")
p([out, err, status.exitstatus()])
p(Open3.capture2e("echo out; echo err 1>&2")[0].split().sort())

# A pipeline joins each command's output to the next one's input.
p(Open3.pipeline("printf 'b\na\n'", "sort").map(&:success?))
Open3.pipeline_r("printf 'x\ny\n'", "wc -l") do |last, waiters|
  p(last.read().strip())
  p(waiters.length())
end
Open3.pipeline_rw("sort", "head -1") do |first, last, _|
  first.puts("zebra", "apple")
  first.close()
  p(last.read())
end
