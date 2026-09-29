# A file handle is an IO, whichever way that is asked.
path = "/tmp/metorex_file_handle_kinds_#{Process.pid}.txt"
File.open(path, "w") do |file|
  p(IO === file)
  p(File === file)
  p(file.is_a?(IO))
  p(file.class)
  kind = case file
         when STDOUT then :stdout
         when IO then :io
         else :other
         end
  p(kind)
end
File.delete(path)
