# Opening shared libraries and finding the symbols in them.
module Fiddle
  class Error < StandardError; end
  class DLError < Error; end

  RTLD_GLOBAL, RTLD_LAZY, RTLD_NOW = __dynamic_library__(:flags)

  # A shared library the program has opened, or the program itself.
  class Handle
    RTLD_GLOBAL = Fiddle::RTLD_GLOBAL
    RTLD_LAZY = Fiddle::RTLD_LAZY
    RTLD_NOW = Fiddle::RTLD_NOW

    def self.sym(name)
      DEFAULT.sym(name)
    end

    def self.[](name)
      sym(name)
    end

    def initialize(library = nil, flags = RTLD_LAZY | RTLD_GLOBAL)
      @address = Fiddle.__dynamic_library__(:open, library, flags)
      @closed = false
      @closable = false
    end

    def sym(name)
      raise DLError, "closed handle" if @closed
      unless name.is_a?(String)
        unless name.respond_to?(:to_str)
          raise TypeError, "no implicit conversion of #{name.class} into String"
        end
        name = name.to_str
      end
      Fiddle.__dynamic_library__(:symbol, @address, name)
    end
    alias [] sym

    def close
      raise DLError, "dlclose() called too many times" if @closed
      @closed = true
      Fiddle.__dynamic_library__(:close, @address)
    end

    def to_i
      @address
    end

    def to_ptr
      @address
    end

    def disable_close
      @closable = false
    end

    def enable_close
      @closable = true
    end

    def close_enabled?
      @closable
    end

    def self.__default__
      made = allocate
      made.instance_variable_set(:@address, Fiddle.__dynamic_library__(:default))
      made.instance_variable_set(:@closed, false)
      made.instance_variable_set(:@closable, false)
      made
    end

    DEFAULT = __default__
  end

  def self.dlopen(library)
    Handle.new(library)
  end
end
