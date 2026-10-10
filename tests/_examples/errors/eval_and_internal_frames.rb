def run_snippet(code) = eval(code, binding, "(snippet)", 7)

begin
  run_snippet("unknown_name")
rescue NameError => error
  p(error.backtrace.first.sub(/\A.*?:/, "FILE:"))
end

begin
  1.tap { raise "inside tap" }
rescue => error
  p(error.backtrace.map { |line| line.sub(/\A[^:]*:/, "FILE:") })
end

class Vault
  private def secret = 1
  protected def guarded = 2
end
p((Vault.new.public_method(:secret) rescue $!))
p((Vault.new.public_method(:guarded) rescue $!))
p((Object.new.public_method(:exit) rescue $!))
