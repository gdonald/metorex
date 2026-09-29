# File.for_fd with no mode uses the descriptor the way it was opened.
path = "/tmp/metorex_file_for_descriptor_#{Process.pid}.txt"
opened = File.open path, "w+"
adopted = File.for_fd opened.fileno
adopted.autoclose = false
p adopted.write("written through the descriptor")
adopted.close
opened.close
p File.read path
File.delete path
