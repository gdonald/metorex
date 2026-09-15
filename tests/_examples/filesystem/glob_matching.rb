# `File.fnmatch` compares a name against a glob pattern without looking at the
# file system, and the flags decide what the wildcards reach.
p File.fnmatch "c?t", "cat"
p File.fnmatch "c*t", "c/a/b/t"
p File.fnmatch "c*t", "c/a/b/t", File::FNM_PATHNAME
p File.fnmatch "ca[a-z]", "cat"
p File.fnmatch "ca[^t]", "cat"
p File.fnmatch "cat", "CAT", File::FNM_CASEFOLD
p File.fnmatch "*", ".profile"
p File.fnmatch "*", ".profile", File::FNM_DOTMATCH
p File.fnmatch "**/*.rb", "one/two/three/main.rb", File::FNM_PATHNAME
p File.fnmatch "c{at,ub}s", "cubs"
p File.fnmatch "c{at,ub}s", "cubs", File::FNM_EXTGLOB
p File.fnmatch "\\?", "?"
p File.fnmatch "\\?", "a"
