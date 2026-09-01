p LoadError.new.path
p LoadError.new("cannot load such file -- widgets").path

begin
  require "file_that_does_not_exist"
rescue LoadError => error
  p error.path
  puts error.message
end

begin
  require_relative "also_missing"
rescue LoadError => error
  puts error.class
  puts error.path.end_with? "also_missing"
end

p RuntimeError.new("boom").respond_to? :path
