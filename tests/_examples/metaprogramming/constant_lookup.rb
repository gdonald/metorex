# Where a constant is found: the scopes a method was written in, then the
# ancestors of the innermost one, then Object and the modules it includes.
# A private constant is read only without its scope written out.

module Shared
  RETRIES = 3
end
include Shared

module Settings
  def self.retries
    RETRIES
  end
end
p(Settings.retries)
p(::RETRIES)

class Base
  LIMIT = :from_base
end
Object::LIMIT = :from_object
class Derived < Base
  def self.limit
    LIMIT
  end
end
p(Derived.limit)

class Lenient
  def self.const_missing(name)
    :"missing_#{name}"
  end

  def self.lookup
    UNDEFINED_HERE
  end
end
p(Lenient.lookup)

anonymous = Module.new do
  def self.name
    nil
  end

  def self.inspect
    "#<Settings module>"
  end
end
begin
  anonymous::ABSENT
rescue NameError => error
  p(error.message)
end

class Vault
  SECRET = :hidden
  private_constant :SECRET

  def self.reveal
    SECRET
  end

  def self.known
    defined?(SECRET)
  end
end
p(Vault.reveal)
p(Vault.known)
p(defined?(Vault::SECRET))
begin
  Vault::SECRET
rescue NameError => error
  p([error.message, error.name, error.receiver])
end

class Vault
  class Inner
  end
  private_constant :Inner
end
begin
  eval("class Vault::Inner; end")
rescue NameError => error
  p(error.message)
end

begin
  eval("def assigns\n  LIMIT = 1\nend")
rescue SyntaxError => error
  p(error.message.include?("dynamic constant assignment"))
end

class Object
  HIDDEN_TOP = :top
  private_constant :HIDDEN_TOP
end
p(HIDDEN_TOP)
begin
  ::HIDDEN_TOP
rescue NameError
  p(HIDDEN_TOP)
end
