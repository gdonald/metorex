# A reference that does not by itself keep an object alive. Metorex frees an
# object when the last reference to it goes, and a WeakRef holds the object
# through ObjectSpace's weak map rather than in a slot of its own.

require 'delegate'

class WeakRef < Delegator
  # Raised when the object a reference was made for is gone.
  class RefError < StandardError
  end

  @@held = ObjectSpace::WeakMap.new

  def initialize(object)
    @weakref_key = Object.new
    @@held[@weakref_key] = object
    super
  end

  def __getobj__
    held = @weakref_key.nil? ? nil : @@held[@weakref_key]
    if held.nil?
      raise RefError, "Invalid Reference - probably recycled"
    end
    held
  end

  def __setobj__(object)
    @@held[@weakref_key] = object unless @weakref_key.nil?
  end

  # Whether the object is still there to be reached.
  def weakref_alive?
    !@weakref_key.nil? && !@@held[@weakref_key].nil?
  end
end
