// Core library pieces defined in Ruby rather than in Rust.
//
// A method written here is a real user-defined method, so it can be aliased,
// redefined, mocked, and introspected the way MRI's own Ruby-level core
// methods can. Kernel#warn relies on that: the specs alias `Warning.warn`
// away and put their own back.

use crate::vm::core::VirtualMachine;

/// Ruby source evaluated into every fresh VM.
const PRELUDE_SOURCE: &str = r##"
module Warning
  def warn(message, category: nil)
    return nil unless category.nil? || Warning[category]
    $stderr.write message
    nil
  end

  extend self
end

# Math wraps the one native primitive in a method per function. Each is a
# module function, so `Math.sqrt` and a private `sqrt` inside a class that
# includes Math both reach it.
module Math
  def sqrt(x)
    __math_function__(:sqrt, x)
  end

  def cbrt(x)
    __math_function__(:cbrt, x)
  end

  def sin(x)
    __math_function__(:sin, x)
  end

  def cos(x)
    __math_function__(:cos, x)
  end

  def tan(x)
    __math_function__(:tan, x)
  end

  def asin(x)
    __math_function__(:asin, x)
  end

  def acos(x)
    __math_function__(:acos, x)
  end

  def atan(x)
    __math_function__(:atan, x)
  end

  def sinh(x)
    __math_function__(:sinh, x)
  end

  def cosh(x)
    __math_function__(:cosh, x)
  end

  def tanh(x)
    __math_function__(:tanh, x)
  end

  def asinh(x)
    __math_function__(:asinh, x)
  end

  def acosh(x)
    __math_function__(:acosh, x)
  end

  def atanh(x)
    __math_function__(:atanh, x)
  end

  def exp(x)
    __math_function__(:exp, x)
  end

  def log2(x)
    __math_function__(:log2, x)
  end

  def log10(x)
    __math_function__(:log10, x)
  end

  def log1p(x)
    __math_function__(:log1p, x)
  end

  def expm1(x)
    __math_function__(:expm1, x)
  end

  def erf(x)
    __math_function__(:erf, x)
  end

  def erfc(x)
    __math_function__(:erfc, x)
  end

  def gamma(x)
    __math_function__(:gamma, x)
  end

  def atan2(y, x)
    __math_function__(:atan2, y, x)
  end

  def hypot(x, y)
    __math_function__(:hypot, x, y)
  end

  def ldexp(fraction, exponent)
    __math_function__(:ldexp, fraction, exponent)
  end

  def frexp(x)
    __math_function__(:frexp, x)
  end

  def lgamma(x)
    __math_function__(:lgamma, x)
  end

  # A base of nil is an argument Ruby refuses, which is not the same as
  # leaving the base out.
  def log(x, *base)
    return __math_function__(:log, x) if base.empty?
    __math_function__(:log, x, base.first)
  end

  module_function :sqrt, :cbrt, :sin, :cos, :tan, :asin, :acos, :atan, :sinh, :cosh, :tanh, :asinh, :acosh, :atanh, :exp, :log2, :log10, :log1p, :expm1, :erf, :erfc, :gamma, :atan2, :hypot, :ldexp, :frexp, :lgamma, :log
end

# Numeric carries the protocol every number answers, written in terms of the
# methods a subclass supplies. Integer and Float reach their own native
# implementations first, so what is here serves the subclasses a program writes.
class Numeric
  def abs
    self < 0 ? -self : self
  end

  def magnitude
    abs
  end

  def ceil(digits = 0)
    to_f.ceil(digits)
  end

  def floor(digits = 0)
    to_f.floor(digits)
  end

  def round(digits = 0)
    to_f.round(digits)
  end

  def truncate(digits = 0)
    to_f.truncate(digits)
  end

  def to_int
    to_i
  end

  def zero?
    self == 0
  end

  def nonzero?
    zero? ? nil : self
  end

  def positive?
    self > 0
  end

  def negative?
    self < 0
  end

  def integer?
    false
  end

  def finite?
    true
  end

  def infinite?
    nil
  end

  def real?
    true
  end

  def real
    self
  end

  def imaginary
    0
  end

  def imag
    imaginary
  end

  def conjugate
    self
  end

  def conj
    conjugate
  end

  def div(other)
    raise ZeroDivisionError, "divided by 0" if other == 0
    (self / other).floor
  end

  def modulo(other)
    self - other * div(other)
  end

  def %(other)
    modulo(other)
  end

  def divmod(other)
    [div(other), modulo(other)]
  end

  # `remainder` truncates the division where `modulo` floors it, so the two
  # differ by one divisor whenever the signs disagree.
  def remainder(other)
    left = self % other
    return left if left == 0
    return left - other if (self < 0 && other > 0) || (self > 0 && other < 0)
    left
  end

  def fdiv(other)
    to_f / other.to_f
  end

  def eql?(other)
    return false unless other.instance_of? self.class
    (self == other) ? true : false
  end

  def coerce(other)
    return [other, self] if other.instance_of? self.class
    [Float(other), Float(self)]
  end

  def numerator
    to_r.numerator
  end

  def denominator
    to_r.denominator
  end

  # Ruby refuses a singleton method on a number, since two numbers of the same
  # value are the same object.
  def singleton_method_added(name)
    raise TypeError, "can't define singleton"
  end

  def dup
    self
  end

  # A number is frozen, so a clone cannot ask for an unfrozen one.
  def clone(freeze: true)
    raise ArgumentError, "can't unfreeze #{self.class}" if freeze == false
    self
  end

  def +@
    self
  end
end

class Complex
  # The imaginary unit, which every other Complex is measured against.
  I = Complex(0, 1)

  # A Complex names no point on the number line, so it answers neither of the
  # questions a real number does.
  undef_method :positive?
  undef_method :negative?
end

class IO
  module WaitReadable
  end

  module WaitWritable
  end

  class EAGAINWaitReadable < Errno::EAGAIN
    include WaitReadable
  end

  class EAGAINWaitWritable < Errno::EAGAIN
    include WaitWritable
  end

  EWOULDBLOCKWaitReadable = EAGAINWaitReadable
  EWOULDBLOCKWaitWritable = EAGAINWaitWritable
end

class StopIteration
  attr_accessor :result
end

class Thread
  class Backtrace
    class Location
      attr_reader :path, :lineno, :label, :absolute_path

      def base_label
        return @label if @label.nil?
        return @label unless @label.start_with?("block ")
        @label.split(" in ", 2).last
      end

      def to_s
        return "#{@path}:#{@lineno}" if @label.nil? || @label.empty?
        "#{@path}:#{@lineno}:in '#{@label}'"
      end

      def inspect
        to_s.inspect
      end
    end
  end
end

# An in-memory IO. `StringIO.new` starts from the string it is given, and
# everything written is appended to it.
class StringIO
  attr_reader :string

  def initialize(string = "")
    @string = string
    @position = 0
  end

  def write(*values)
    written = 0
    values.each do |value|
      text = value.to_s
      @string = @string + text
      written = written + text.length
    end
    written
  end

  def <<(value)
    write value
    self
  end

  def print(*values)
    values.each { |value| write value }
    nil
  end

  def printf(format, *values)
    write format % values
    nil
  end

  def puts(*values)
    if values.empty?
      write "\n"
      return nil
    end
    values.each do |value|
      text = value.to_s
      write text
      write "\n" unless text.end_with? "\n"
    end
    nil
  end

  def read(length = nil)
    remaining = @string[@position..-1] || ""
    taken = length.nil? ? remaining : remaining[0, length]
    @position = @position + taken.length
    taken
  end

  def gets
    remaining = @string[@position..-1] || ""
    return nil if remaining.empty?
    break_at = nil
    position = 0
    while position < remaining.length
      if remaining[position] == "\n"
        break_at = position
        break
      end
      position = position + 1
    end
    line = break_at.nil? ? remaining : remaining[0, break_at + 1]
    @position = @position + line.length
    line
  end

  def rewind
    @position = 0
    0
  end

  def close
    nil
  end

  def closed?
    false
  end

  def to_s
    @string
  end
end

class Enumerator
  include Enumerable

  # Collects what a generator block hands it, so `Enumerator.new { |y| y << 1 }`
  # reads the same way as one built over a method that yields.
  class Yielder
    def initialize(collected)
      @collected = collected
    end

    def <<(value)
      @collected.push(value)
      self
    end

    def yield(*values)
      @collected.push(values.size == 1 ? values[0] : values)
      nil
    end
  end

  def initialize(receiver = nil, method_name = nil, arguments = [], size = nil, &generator)
    @receiver = receiver
    @method_name = method_name
    @arguments = arguments
    @size = size
    @position = 0
    @generator = generator
  end

  def size
    @size
  end

  def to_a
    if @values.nil?
      collected = []
      if @generator.nil?
        @result = @receiver.send(@method_name, *@arguments) do |*yielded|
          collected.push(yielded.size == 1 ? yielded[0] : yielded)
        end
      else
        @result = @generator.call(Yielder.new(collected))
      end
      @values = collected
    end
    @values
  end

  def peek
    values = to_a
    if @position >= values.size
      ended = StopIteration.new("iteration reached an end")
      ended.result = @result
      raise ended
    end
    values[@position]
  end

  def next
    value = peek
    @position = @position + 1
    value
  end

  def rewind
    @position = 0
    self
  end

  # The enumerator `loop` answers when called without a block: it yields
  # forever and reports an endless size.
  def self.endless
    enumerator = new(nil, nil, [], Float::INFINITY)
    enumerator.mark_endless
  end

  def mark_endless
    @endless = true
    self
  end

  def each(&block)
    return self if block.nil?
    if @endless
      while true
        block.call
      end
    end
    to_a.each { |value| block.call(value) }
    @receiver
  end

  def first(count = nil)
    return to_a[0] if count.nil?
    to_a[0, count]
  end

  def map(&block)
    return self if block.nil?
    to_a.map { |value| block.call(value) }
  end

  def collect(&block)
    map(&block)
  end

  def inspect
    "#<Enumerator: #{@receiver.inspect}:#{@method_name}>"
  end
end
"##;

impl VirtualMachine {
    /// Build an `Enumerator` over `method_name` sent to `receiver`, which is
    /// what a method that yields answers when called without a block.
    pub(crate) fn build_enumerator(
        &mut self,
        receiver: crate::object::Object,
        method_name: &str,
        arguments: Vec<crate::object::Object>,
        size: Option<i64>,
        position: crate::lexer::Position,
    ) -> Result<crate::object::Object, crate::error::MetorexError> {
        use crate::object::Object;
        let Some(enumerator_class) = self.globals().get("Enumerator") else {
            let message = "uninitialized constant Enumerator".to_string();
            return Err(crate::error::MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let arguments = vec![
            receiver,
            Object::Symbol(std::rc::Rc::new(method_name.to_string())),
            Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arguments))),
            match size {
                Some(size) => Object::Int(size),
                None => Object::Nil,
            },
        ];
        self.send_to_object(enumerator_class, "new", arguments, position)
    }

    /// Evaluate the Ruby-level core library. A parse or runtime failure here
    /// is a defect in `PRELUDE_SOURCE` itself, so it panics rather than
    /// leaving a half-built VM behind.
    pub(crate) fn load_prelude(&mut self) {
        let tokens = crate::lexer::Lexer::new(PRELUDE_SOURCE).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .unwrap_or_else(|errors| panic!("prelude failed to parse: {:?}", errors));
        self.execute_program(&statements)
            .unwrap_or_else(|error| panic!("prelude failed to run: {}", error));
    }
}
