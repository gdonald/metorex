# An object whose fields are decided as they are set, rather than by a class
# written ahead of time.
class OpenStruct
  def initialize(fields = nil)
    @table = {}
    return if fields.nil?
    fields.each_pair { |name, value| @table[name.to_sym] = value }
  end

  private :initialize

  def [](name)
    @table[name.to_sym]
  end

  def []=(name, value)
    refuse_when_frozen
    @table[name.to_sym] = value
  end

  def delete_field(name)
    named = name.to_sym
    unless @table.key?(named)
      return yield(named) if block_given?
      raise NameError, "no field '#{named}' in #{self}"
    end
    @table.delete(named)
  end

  def dig(name, *rest)
    held = @table[name.to_sym]
    return held if rest.empty? || held.nil?
    held.dig(*rest)
  end

  def each_pair
    return to_enum(:each_pair) unless block_given?
    @table.each_pair { |name, value| yield name, value }
    self
  end

  def to_h(&block)
    return @table.to_h(&block) unless block.nil?
    copied = {}
    @table.each_pair { |name, value| copied[name] = value }
    copied
  end

  def respond_to_missing?(name, include_private = false)
    spelling = name.to_s
    spelling = spelling[0, spelling.length - 1] if spelling.end_with?("=")
    @table.key?(spelling.to_sym)
  end

  def method_missing(name, *args)
    spelling = name.to_s
    if spelling.end_with?("=")
      unless args.length == 1
        raise ArgumentError, "wrong number of arguments (given #{args.length}, expected 1)"
      end
      refuse_when_frozen
      return @table[spelling[0, spelling.length - 1].to_sym] = args[0]
    end
    named = spelling.to_sym
    unless args.empty?
      if @table.key?(named)
        raise ArgumentError, "wrong number of arguments (given #{args.length}, expected 0)"
      end
      raise NoMethodError, "undefined method '#{spelling}' for an instance of #{self.class}"
    end
    @table[named]
  end

  def ==(other)
    return false unless other.is_a?(OpenStruct)
    to_h == other.to_h
  end

  def eql?(other)
    return false unless other.is_a?(OpenStruct)
    to_h.eql?(other.to_h)
  end

  def hash
    @table.hash
  end

  def marshal_dump
    to_h
  end

  def marshal_load(fields)
    @table = {}
    fields.each_pair { |name, value| @table[name.to_sym] = value }
    self
  end

  def inspect
    render([])
  end

  def to_s
    render([])
  end

  # What a rendering shows: the class name and every field, where a field
  # that reaches back to this object is written as the class alone.
  def render(walking)
    label = self.class.name
    return "#<#{label} ...>" if walking.any? { |held| held.equal?(self) }
    walking.push(self)
    parts = @table.map do |name, value|
      shown = value.is_a?(OpenStruct) ? value.render(walking) : value.inspect
      "#{name}=#{shown}"
    end
    walking.pop
    parts.empty? ? "#<#{label}>" : "#<#{label} #{parts.join(", ")}>"
  end

  # A frozen object holds the fields it had, and nothing may be written to it.
  def refuse_when_frozen
    raise FrozenError, "can't modify frozen #{self.class}: #{self}" if frozen?
  end
  private :refuse_when_frozen
end
