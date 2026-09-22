pub(super) const SOURCE: &str = r##"
class IO
  # `putc` writes one character: the first of a String, or the low byte of a
  # number. It answers what it was given rather than what it wrote.
  def putc(held)
    if held.is_a? String
      write held[0]
      return held
    end
    number = if held.is_a? Integer
      held
    elsif held.respond_to? :to_int
      held.to_int
    else
      raise TypeError, "no implicit conversion of #{held.class} into Integer"
    end
    write (number & 0xFF).chr
    held
  end
end

class IO
  # A run of bytes a program reads and writes directly. A buffer either holds
  # memory of its own or stands over a String or a file, and a slice of one
  # shares the bytes it was cut from.
  class Buffer
    PAGE_SIZE = 4096
    DEFAULT_SIZE = 65536

    EXTERNAL = 1
    INTERNAL = 2
    MAPPED = 4
    SHARED = 8
    LOCKED = 32
    PRIVATE = 64
    READONLY = 128

    class LockedError < RuntimeError
    end

    class AllocationError < RuntimeError
    end

    class AccessError < RuntimeError
    end

    class InvalidatedError < RuntimeError
    end

    class MaskError < ArgumentError
    end

    # A size or an offset arrives as an Integer and nothing else.
    def self.whole_number(held)
      raise TypeError, "not an Integer" unless held.is_a? Integer
      if held > 9223372036854775807 || held < -9223372036854775808
        raise RangeError, "bignum too big to convert into `long'"
      end
      held
    end

    # The text a run of bytes stands for. Bytes that spell characters in UTF-8
    # read back as those characters, and bytes that spell nothing stand for
    # themselves.
    def self.text_of(bytes)
      return "".b if bytes.empty?
      bytes.pack "C*"
    end

    # Without flags the buffer picks where its bytes live by how many there
    # are. Flags that name neither place leave it nowhere to put them.
    def initialize(size = DEFAULT_SIZE, flags = nil)
      size = IO::Buffer.whole_number size
      flags = IO::Buffer.whole_number(flags) unless flags.nil?
      raise ArgumentError, "Size can't be negative!" if size < 0
      raise ArgumentError, "Flags can't be negative!" if !flags.nil? && flags < 0
      @locked = false
      @source = nil
      @string_backed = false
      if size == 0
        nullify
        return
      end
      kind = if flags.nil?
        size < PAGE_SIZE ? INTERNAL : MAPPED
      elsif (flags & MAPPED) != 0
        MAPPED
      elsif (flags & INTERNAL) != 0
        INTERNAL
      else
        raise AllocationError, "Could not allocate buffer!"
      end
      @text = "\0" * size
      @offset = 0
      @size = size
      @flags = kind | ((flags || 0) & (SHARED | PRIVATE | READONLY))
    end

    # A buffer over a String. Without a block the bytes are copied and the
    # copy is read only, and with one the String itself is written through
    # and left alone until the block ends.
    def self.for(string)
      unless block_given?
        made = IO::Buffer.new 0
        made.send :adopt_string, string.bytes.pack("C*"), EXTERNAL | READONLY
        return made
      end
      made = IO::Buffer.new 0
      made.send :adopt_string, string, EXTERNAL | (string.frozen? ? READONLY : 0)
      string.__borrow__ unless string.frozen?
      begin
        yield made
      ensure
        string.__release__ unless string.frozen?
        made.free unless made.null?
      end
    end

    # A buffer over a String of the size asked for, which is answered once the
    # block is done with it.
    def self.string(length)
      raise LocalJumpError, "no block given" unless block_given?
      length = IO::Buffer.whole_number length
      raise ArgumentError, "negative string size (or size too big)" if length < 0
      held = "\0" * length
      made = IO::Buffer.new 0
      made.send :adopt_string, held, EXTERNAL
      begin
        yield made
      ensure
        made.free unless made.null?
      end
      held
    end

    # A buffer over what a file holds.
    def self.map(file, size = nil, offset = 0, flags = 0)
      offset = IO::Buffer.whole_number offset
      flags = IO::Buffer.whole_number flags
      raise ArgumentError, "Offset can't be negative!" if offset < 0
      content = File.read(file.path).b
      whole = content.bytesize
      raise ArgumentError, "Invalid negative or zero file size!" if whole == 0
      unless size.nil?
        size = IO::Buffer.whole_number size
        raise ArgumentError, "Size can't be negative!" if size < 0
        raise ArgumentError, "Size can't be zero!" if size == 0
        raise ArgumentError, "Size can't be larger than file size!" if size > whole
        raise ArgumentError, "Offset too large!" if offset + size > whole
      end
      size = whole - offset if size.nil?
      bytes = content.bytes[offset, size] || []
      made = IO::Buffer.new 0
      kind = if (flags & PRIVATE) != 0
        MAPPED | PRIVATE
      else
        MAPPED | EXTERNAL | SHARED
      end
      made.send :adopt_string, IO::Buffer.text_of(bytes), kind | (flags & READONLY)
      made.send :follow_file, file, offset if (flags & PRIVATE) == 0
      made
    end

    def size
      @size
    end

    def empty?
      @size == 0
    end

    def null?
      @text.nil?
    end

    def external?
      !null? && (@flags & EXTERNAL) != 0
    end

    def internal?
      !null? && (@flags & INTERNAL) != 0
    end

    def mapped?
      !null? && (@flags & MAPPED) != 0
    end

    def shared?
      !null? && (@flags & SHARED) != 0
    end

    def private?
      !null? && (@flags & PRIVATE) != 0
    end

    def readonly?
      !null? && (@flags & READONLY) != 0
    end

    def locked?
      @locked == true
    end

    # A buffer under a lock refuses every change to itself, though what it
    # holds is still read and written.
    def locked
      raise LockedError, "Buffer already locked!" if @locked
      @locked = true
      begin
        yield
      ensure
        @locked = false
      end
    end

    # A slice is valid while what it was cut from is still there and still
    # covers it.
    def valid?
      return true if @source.nil?
      return true if @source_string_backed
      return false if @source.null?
      return false unless @source.send(:holds_text?, @source_text)
      @offset + @size <= @source.send(:end_offset)
    end

    def free
      raise LockedError, "Buffer is locked!" if @locked
      nullify
      self
    end

    def transfer
      raise LockedError, "Cannot transfer ownership of locked buffer!" if @locked
      made = IO::Buffer.new 0
      made.send :adopt, @text, @offset, @size, @flags, @source, @source_text,
                @source_string_backed, @file, @file_offset
      nullify
      made
    end

    def slice(at = 0, length = nil)
      ensure_valid
      at = IO::Buffer.whole_number at
      length = length.nil? ? @size - at : IO::Buffer.whole_number(length)
      made = IO::Buffer.new 0
      made.send :adopt, @text, @offset + at, length, @flags & ~READONLY, self,
                @text, @string_backed, nil, 0
      made
    end

    def resize(size)
      raise LockedError, "Cannot resize locked buffer!" if @locked
      size = IO::Buffer.whole_number size
      raise ArgumentError, "Size can't be negative!" if size < 0
      raise AccessError, "Cannot resize external buffer!" if external?
      if size == 0
        nullify
        return self
      end
      held = null? ? [] : byte_view
      made = Array.new size, 0
      counted = size < held.length ? size : held.length
      counted.times { |at| made[at] = held[at] }
      kind = if null?
        size < PAGE_SIZE ? INTERNAL : MAPPED
      elsif mapped? && !private?
        # Linux resizes the mapping in place, so the buffer stays mapped.
        # Elsewhere the bytes are copied into one of the program's own.
        RUBY_PLATFORM.include?("linux") ? MAPPED : INTERNAL
      else
        @flags & (INTERNAL | MAPPED)
      end
      @text = IO::Buffer.text_of made
      @offset = 0
      @size = size
      @flags = kind | (@flags & (SHARED | PRIVATE | READONLY))
      self
    end

    def get_string(at = 0, length = nil, encoding = nil)
      ensure_valid
      at = IO::Buffer.whole_number at
      length = length.nil? ? @size - at : IO::Buffer.whole_number(length)
      taken = byte_view[at, length] || []
      IO::Buffer.text_of taken
    end

    def set_string(text, at = 0, length = nil, source_offset = 0)
      ensure_valid
      raise AccessError, "Buffer is not writable!" if readonly?
      at = IO::Buffer.whole_number at
      source_offset = IO::Buffer.whole_number source_offset
      given = text.bytes
      given = given[source_offset..-1] || []
      given = given[0, length] || [] unless length.nil?
      room = @size - at
      given = given[0, room] || [] if given.length > room
      set_bytes given, at
      given.length
    end

    def clear(value = 0, at = 0, length = nil)
      ensure_valid
      raise AccessError, "Buffer is not writable!" if readonly?
      length = length.nil? ? @size - at : IO::Buffer.whole_number(length)
      set_bytes Array.new(length, value), at
      self
    end

    def each(kind = :U8)
      held = byte_view
      unless block_given?
        made = []
        at = 0
        while at < held.length
          made.push [at, held[at]]
          at = at + 1
        end
        return made.each
      end
      at = 0
      while at < held.length
        yield at, held[at]
        at = at + 1
      end
      self
    end

    def &(mask)
      combined mask, :and, false
    end

    def |(mask)
      combined mask, :or, false
    end

    def ^(mask)
      combined mask, :xor, false
    end

    def ~
      combined nil, :invert, false
    end

    def and!(mask)
      combined mask, :and, true
    end

    def or!(mask)
      combined mask, :or, true
    end

    def xor!(mask)
      combined mask, :xor, true
    end

    def not!
      combined nil, :invert, true
    end

    def to_s
      return "#<IO::Buffer 0x0000000000000000 +0 0 NULL>" if null?
      "#<IO::Buffer 0x%016x +%d %d %s>" % [@text.object_id, @offset, @size, flag_names]
    end

    def inspect
      to_s
    end

    def hexdump(at = 0, length = nil, width = 16)
      taken = byte_view[at, length.nil? ? @size - at : length] || []
      taken.map { |value| "%02x" % value }.join(" ")
    end

    # ── What a buffer keeps to itself ────────────────────────────────────────

    def adopt(text, offset, size, flags, source, source_text, source_string_backed, file, file_offset)
      @text = text
      @offset = offset
      @size = size
      @flags = flags
      @source = source
      @source_text = source_text
      @source_string_backed = source_string_backed
      @string_backed = source_string_backed
      @file = file
      @file_offset = file_offset
      @locked = false
      self
    end

    def adopt_string(text, flags)
      @text = text
      @offset = 0
      @size = text.bytesize
      @flags = flags
      @source = nil
      @source_text = nil
      @source_string_backed = false
      @string_backed = true
      @locked = false
      self
    end

    def follow_file(file, offset)
      @file = file
      @file_offset = offset
      @string_backed = false
      self
    end

    def nullify
      @text = nil
      @offset = 0
      @size = 0
      @flags = 0
      @file = nil
      @file_offset = 0
    end

    def holds_text?(other)
      !@text.nil? && @text.equal?(other)
    end

    def end_offset
      @offset + @size
    end

    def byte_view
      return [] if @text.nil?
      @text.bytes[@offset, @size] || []
    end

    def ensure_valid
      raise InvalidatedError, "Buffer has been invalidated!" unless valid?
    end

    def set_bytes(values, at = 0)
      held = @text.bytes
      values.each_with_index do |value, step|
        place = @offset + at + step
        held[place] = value & 0xff if place < @offset + @size
      end
      made = IO::Buffer.text_of held
      # The buffer is allowed to write through the very lock it put on the
      # string it stands over.
      @text.__release__
      @text.replace made
      @text.__borrow__ if @string_backed && !@source.nil? == false && borrowing?
      write_through
      self
    end

    def borrowing?
      false
    end

    def write_through
      return if @file.nil?
      return unless shared?
      File.write @file.path, IO::Buffer.text_of(@text.bytes)
    end

    def flag_names
      named = []
      named << "EXTERNAL" if external?
      named << "INTERNAL" if internal?
      named << "MAPPED" if mapped?
      named << "SHARED" if shared?
      named << "PRIVATE" if private?
      named << "READONLY" if readonly?
      named.empty? ? "NULL" : named.join("|")
    end

    def combined(mask, how, in_place)
      ensure_valid
      values = byte_view
      made = if how == :invert
        values.map { |value| ~value & 0xff }
      else
        unless mask.is_a? IO::Buffer
          named = mask.nil? ? "nil" : mask.class.to_s
          raise TypeError, "wrong argument type #{named} (expected IO::Buffer)"
        end
        over = mask.send :byte_view
        raise MaskError, "Zero-length mask given!" if over.empty?
        values.each_with_index.map do |value, at|
          other = over[at % over.length]
          if how == :and
            value & other
          elsif how == :or
            value | other
          else
            value ^ other
          end
        end
      end
      if in_place
        set_bytes made
        return self
      end
      answer = IO::Buffer.new made.length, INTERNAL
      answer.send :set_bytes, made
      answer
    end

    private :adopt, :adopt_string, :follow_file, :nullify, :holds_text?,
            :end_offset, :byte_view, :ensure_valid, :set_bytes, :borrowing?,
            :write_through, :flag_names, :combined
  end
end
"##;
