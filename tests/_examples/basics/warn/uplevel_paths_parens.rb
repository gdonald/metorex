# A warning with `uplevel:` names the file the way it was given to ruby.
$stderr = $stdout

def caution
  warn("from the caller", uplevel: 1)
end

caution
warn("from here", uplevel: 0)
