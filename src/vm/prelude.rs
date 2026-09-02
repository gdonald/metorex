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
  def self.warn(message, category: nil)
    $stderr.write message
    nil
  end
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
