# A Hash before the command names what the child's environment adds, or takes
# away where a name is given nil, and leaves the parent's own alone.
$stdout.sync = true
ENV["KEPT"] = "parent"
p system({ "ADDED" => "child" }, "sh", "-c", 'echo "$ADDED $KEPT"')
p system({ "KEPT" => nil }, "sh", "-c", 'echo "[$KEPT]"')
p system({ "ADDED" => "shell" }, 'echo "$ADDED"')
p system ["sh", "named"], "-c", 'echo $0'
p system({ "CODE" => "4" }, 'exit $CODE')
p $?.exitstatus
p ENV["ADDED"]
