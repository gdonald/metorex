# Rectangular arrays of numbers and the arithmetic over them.
module ExceptionForMatrix
  class ErrDimensionMismatch < StandardError
  end

  class ErrNotRegular < StandardError
  end

  class ErrOperationNotDefined < StandardError
  end

  class ErrOperationNotImplemented < StandardError
  end
end

class Vector
  include Enumerable

  ErrDimensionMismatch = ExceptionForMatrix::ErrDimensionMismatch
  ErrOperationNotDefined = ExceptionForMatrix::ErrOperationNotDefined
  ErrOperationNotImplemented = ExceptionForMatrix::ErrOperationNotImplemented
  ZERO = nil

  def self.[](*elements)
    made = allocate
    made.send(:build_from, elements)
    made
  end

  def self.elements(list, copy = true)
    self[*list.to_a]
  end

  def self.zero(count)
    self[*Array.new(count, 0)]
  end

  def self.basis(size:, index:)
    unless index >= 0 && index < size
      raise ArgumentError, "invalid index"
    end
    values = Array.new(size, 0)
    values[index] = 1
    self[*values]
  end

  def self.independent?(*vectors)
    vectors.each do |held|
      raise TypeError, "expected Vector, got #{held.class}" unless held.is_a?(Vector)
      unless held.size == vectors.first.size
        raise ExceptionForMatrix::ErrDimensionMismatch
      end
    end
    return false if vectors.count > vectors.first.size
    Matrix[*vectors.map { |held| held.to_a }].rank == vectors.count
  end

  def build_from(elements)
    @elements = elements
    self
  end
  private :build_from

  def to_a
    @elements.dup
  end

  def size
    @elements.size
  end

  def [](index)
    @elements[index]
  end

  def element(index)
    @elements[index]
  end

  def each(&block)
    return to_enum(:each) if block.nil?
    @elements.each(&block)
    self
  end

  def each2(other)
    raise TypeError, "expected Vector" unless other.is_a?(Vector)
    raise ExceptionForMatrix::ErrDimensionMismatch unless size == other.size
    return to_enum(:each2, other) unless block_given?
    index = 0
    while index < size
      yield @elements[index], other[index]
      index += 1
    end
    self
  end

  def collect2(other)
    raise TypeError, "expected Vector" unless other.is_a?(Vector)
    raise ExceptionForMatrix::ErrDimensionMismatch unless size == other.size
    return to_enum(:collect2, other) unless block_given?
    collected = []
    index = 0
    while index < size
      collected.push(yield(@elements[index], other[index]))
      index += 1
    end
    collected
  end

  def collect(&block)
    return to_enum(:collect) if block.nil?
    Vector[*@elements.map(&block)]
  end

  def map(&block)
    collect(&block)
  end

  def +(other)
    return Vector[*collect2(other) { |mine, theirs| mine + theirs }] if other.is_a?(Vector)
    return covector.t + other if other.is_a?(Matrix)
    raise TypeError, "wrong argument type #{other.class}"
  end

  def -(other)
    return Vector[*collect2(other) { |mine, theirs| mine - theirs }] if other.is_a?(Vector)
    return covector.t - other if other.is_a?(Matrix)
    raise TypeError, "wrong argument type #{other.class}"
  end

  def *(other)
    return Vector[*@elements.map { |value| value * other }] if other.is_a?(Numeric)
    return Matrix.column_vector(@elements) * other if other.is_a?(Matrix)
    raise ExceptionForMatrix::ErrOperationNotDefined if other.is_a?(Vector)
    raise TypeError, "wrong argument type #{other.class}"
  end

  def /(other)
    return Vector[*@elements.map { |value| value / other }] if other.is_a?(Numeric)
    raise TypeError, "wrong argument type #{other.class}"
  end

  def ==(other)
    return false unless other.is_a?(Vector)
    return false unless size == other.size
    (0...size).all? { |index| @elements[index] == other[index] }
  end

  def eql?(other)
    return false unless other.is_a?(Vector)
    to_a.eql?(other.to_a)
  end

  def hash
    @elements.hash
  end

  def inner_product(other)
    raise ExceptionForMatrix::ErrDimensionMismatch unless size == other.size
    total = 0
    index = 0
    while index < size
      total += @elements[index] * other[index]
      index += 1
    end
    total
  end

  def dot(other)
    inner_product(other)
  end

  def cross_product(*others)
    unless size >= 2
      raise ExceptionForMatrix::ErrOperationNotDefined
    end
    if size == 3
      raise ExceptionForMatrix::ErrDimensionMismatch unless others.size == 1
      other = others[0]
      raise ExceptionForMatrix::ErrDimensionMismatch unless other.size == 3
      first = @elements[1] * other[2] - @elements[2] * other[1]
      second = @elements[2] * other[0] - @elements[0] * other[2]
      third = @elements[0] * other[1] - @elements[1] * other[0]
      return Vector[first, second, third]
    end
    if size == 2 && others.empty?
      return Vector[-@elements[1], @elements[0]]
    end
    raise ExceptionForMatrix::ErrDimensionMismatch
  end

  def cross(*others)
    cross_product(*others)
  end

  def magnitude
    Math.sqrt(@elements.inject(0) { |total, value| total + value * value })
  end

  def norm
    magnitude
  end

  def r
    magnitude
  end

  def normalize
    reach = magnitude
    raise Vector::ZeroVectorError, "Zero vectors can not be normalized." if reach == 0
    self / reach
  end

  def zero?
    @elements.all? { |value| value == 0 }
  end

  def covector
    Matrix.row_vector(@elements)
  end

  def to_matrix
    Matrix.column_vector(@elements)
  end

  def angle_with(other)
    raise TypeError, "expected Vector" unless other.is_a?(Vector)
    raise ExceptionForMatrix::ErrDimensionMismatch unless size == other.size
    if zero? || other.zero?
      raise Vector::ZeroVectorError, "Can't get angle of zero vector"
    end
    Math.acos(inner_product(other) / (magnitude * other.magnitude))
  end

  def inspect
    "Vector[#{@elements.map { |value| value.inspect }.join(", ")}]"
  end

  def to_s
    "Vector[#{@elements.map { |value| value.to_s }.join(", ")}]"
  end

  class ZeroVectorError < StandardError
  end
end

class Matrix
  include Enumerable

  ErrDimensionMismatch = ExceptionForMatrix::ErrDimensionMismatch
  ErrNotRegular = ExceptionForMatrix::ErrNotRegular
  ErrOperationNotDefined = ExceptionForMatrix::ErrOperationNotDefined
  ErrOperationNotImplemented = ExceptionForMatrix::ErrOperationNotImplemented

  def self.[](*rows)
    made = allocate
    made.send(:build_from, rows.map { |row| as_row(row) })
    made
  end

  # One row of a matrix, taken from an Array, a Vector, or anything that
  # answers to `to_ary`.
  def self.as_row(row)
    return row.to_a if row.is_a?(Vector)
    return row if row.is_a?(Array)
    unless row.respond_to?(:to_ary)
      raise TypeError, "expected Array, got #{row.class}"
    end
    converted = row.to_ary
    raise TypeError, "expected Array, got #{row.class}" unless converted.is_a?(Array)
    converted
  end
  private_class_method :as_row

  def self.rows(rows, copy = true)
    self[*rows.to_a]
  end

  def self.columns(columns)
    rows(columns.to_a).transpose
  end

  def self.build(row_count, column_count = row_count)
    raise ArgumentError, "negative size" if row_count < 0 || column_count < 0
    return to_enum(:build, row_count, column_count) unless block_given?
    made = (0...row_count).map do |row|
      (0...column_count).map { |column| yield row, column }
    end
    rows(made)
  end

  def self.diagonal(*values)
    values = values[0].to_a if values.size == 1 && values[0].is_a?(Array)
    count = values.size
    made = (0...count).map do |row|
      (0...count).map { |column| row == column ? values[row] : 0 }
    end
    rows(made)
  end

  def self.scalar(count, value)
    diagonal(*Array.new(count, value))
  end

  def self.identity(count)
    scalar(count, 1)
  end

  def self.unit(count)
    identity(count)
  end

  def self.I(count)
    identity(count)
  end

  def self.zero(row_count, column_count = row_count)
    rows((0...row_count).map { Array.new(column_count, 0) })
  end

  def self.row_vector(values)
    rows([values.to_a])
  end

  def self.column_vector(values)
    rows(values.to_a.map { |value| [value] })
  end

  def self.empty(row_count = 0, column_count = 0)
    if row_count != 0 && column_count != 0
      raise ArgumentError, "One size must be 0, got #{row_count}x#{column_count}"
    end
    raise ArgumentError, "negative size" if row_count < 0 || column_count < 0
    made = allocate
    made.send(:build_from, (0...row_count).map { [] }, column_count)
    made
  end

  def build_from(rows, column_count = nil)
    @rows = rows
    @column_count = if column_count.nil?
      rows.empty? ? 0 : rows[0].length
    else
      column_count
    end
    rows.each do |row|
      raise ExceptionForMatrix::ErrDimensionMismatch unless row.length == @column_count
    end
    self
  end
  private :build_from

  # ── Shape ────────────────────────────────────────────────────────────────

  def row_size
    @rows.size
  end

  def row_count
    @rows.size
  end

  def column_size
    @column_count
  end

  def column_count
    @column_count
  end

  def to_a
    @rows.map { |row| row.dup }
  end

  def [](row, column)
    held = @rows[row]
    return nil if held.nil?
    return nil if column >= @column_count || column < -@column_count
    held[column]
  end

  def element(row, column)
    self[row, column]
  end

  def row(index, &block)
    held = @rows[index]
    if block.nil?
      return nil if held.nil?
      return Vector[*held]
    end
    held.each(&block) unless held.nil?
    self
  end

  def column(index, &block)
    if index >= @column_count || index < -@column_count
      return nil if block.nil?
      return self
    end
    values = @rows.map { |row| row[index] }
    return Vector[*values] if block.nil?
    values.each(&block)
    self
  end

  def row_vectors
    (0...row_size).map { |index| row(index) }
  end

  def column_vectors
    (0...column_size).map { |index| column(index) }
  end

  def each(which = :all, &block)
    return to_enum(:each, which) if block.nil?
    @rows.each_with_index do |row, row_index|
      row.each_with_index do |value, column_index|
        block.call(value) if wanted?(which, row_index, column_index)
      end
    end
    self
  end

  def each_with_index(which = :all)
    return to_enum(:each_with_index, which) unless block_given?
    @rows.each_with_index do |row, row_index|
      row.each_with_index do |value, column_index|
        yield value, row_index, column_index if wanted?(which, row_index, column_index)
      end
    end
    self
  end

  # Whether a place is among the ones a walk was asked for.
  def wanted?(which, row_index, column_index)
    return true if which == :all
    return row_index == column_index if which == :diagonal
    return row_index != column_index if which == :off_diagonal
    return row_index <= column_index if which == :upper
    return row_index < column_index if which == :strict_upper
    return row_index > column_index if which == :strict_lower
    return row_index >= column_index if which == :lower
    raise ArgumentError, "expected #{which.inspect} to be one of :all, :diagonal, :off_diagonal, :lower, :strict_lower, :strict_upper or :upper"
  end
  private :wanted?

  def collect(which = :all, &block)
    return to_enum(:collect, which) if block.nil?
    made = @rows.map { |row| row.map(&block) }
    new_matrix(made, @column_count)
  end

  def map(which = :all, &block)
    collect(which, &block)
  end

  def find_index(*args)
    return to_enum(:find_index) if args.empty? && !block_given?
    wanted = args.first
    @rows.each_with_index do |row, row_index|
      row.each_with_index do |value, column_index|
        matched = args.empty? ? yield(value) : value == wanted
        return [row_index, column_index] if matched
      end
    end
    nil
  end

  def index(*args, &block)
    find_index(*args, &block)
  end

  def minor(*args)
    if args.size == 2
      rows_wanted = args[0]
      columns_wanted = args[1]
      picked = @rows[rows_wanted] || []
      made = picked.map { |row| row[columns_wanted] || [] }
      return new_matrix(made)
    end
    unless args.size == 4
      raise ArgumentError, "wrong number of arguments (given #{args.size}, expected 2 or 4)"
    end
    made = (args[0]...(args[0] + args[1])).map do |row|
      (args[2]...(args[2] + args[3])).map { |column| @rows[row][column] }
    end
    new_matrix(made)
  end

  def first_minor(row, column)
    unless square? || true
      raise RuntimeError
    end
    if row >= row_size || column >= column_size || row < 0 || column < 0
      raise ArgumentError, "index out of range"
    end
    made = []
    @rows.each_with_index do |held, row_index|
      next if row_index == row
      kept = []
      held.each_with_index do |value, column_index|
        kept.push(value) unless column_index == column
      end
      made.push(kept)
    end
    new_matrix(made)
  end

  def transpose
    return self.class.empty(column_size, row_size) if @rows.empty?
    made = (0...@column_count).map do |column|
      @rows.map { |row| row[column] }
    end
    new_matrix(made)
  end

  def t
    transpose
  end

  # ── Arithmetic ───────────────────────────────────────────────────────────

  def +(other)
    raise ExceptionForMatrix::ErrOperationNotDefined if other.is_a?(Numeric)
    other = other.covector.t if other.is_a?(Vector)
    raise TypeError, "wrong argument type #{other.class}" unless other.is_a?(Matrix)
    unless row_size == other.row_size && column_size == other.column_size
      raise ExceptionForMatrix::ErrDimensionMismatch
    end
    made = (0...row_size).map do |row|
      (0...column_size).map { |column| self[row, column] + other[row, column] }
    end
    new_matrix(made)
  end

  def -(other)
    raise ExceptionForMatrix::ErrOperationNotDefined if other.is_a?(Numeric)
    other = other.covector.t if other.is_a?(Vector)
    raise TypeError, "wrong argument type #{other.class}" unless other.is_a?(Matrix)
    unless row_size == other.row_size && column_size == other.column_size
      raise ExceptionForMatrix::ErrDimensionMismatch
    end
    made = (0...row_size).map do |row|
      (0...column_size).map { |column| self[row, column] - other[row, column] }
    end
    new_matrix(made)
  end

  def *(other)
    if other.is_a?(Numeric)
      return new_matrix(@rows.map { |row| row.map { |value| value * other } })
    end
    if other.is_a?(Vector)
      answered = self * other.covector.t
      return Vector[*answered.column(0).to_a]
    end
    raise TypeError, "wrong argument type #{other.class}" unless other.is_a?(Matrix)
    raise ExceptionForMatrix::ErrDimensionMismatch unless column_size == other.row_size
    made = (0...row_size).map do |row|
      (0...other.column_size).map do |column|
        total = 0
        (0...column_size).each { |step| total += self[row, step] * other[step, column] }
        total
      end
    end
    new_matrix(made)
  end

  def /(other)
    if other.is_a?(Numeric)
      return new_matrix(@rows.map { |row| row.map { |value| value / other } })
    end
    return self * other.inverse if other.is_a?(Matrix)
    raise TypeError, "wrong argument type #{other.class}"
  end

  def **(count)
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    unless count.is_a?(Integer)
      raise ExceptionForMatrix::ErrOperationNotImplemented
    end
    return inverse ** -count if count < 0
    answered = Matrix.identity(row_size)
    count.times { answered = answered * self }
    answered
  end

  def -@
    new_matrix(@rows.map { |row| row.map { |value| -value } })
  end

  def +@
    self
  end

  # ── What a matrix answers about itself ───────────────────────────────────

  def square?
    row_size == column_size
  end

  def empty?
    row_size == 0 || column_size == 0
  end

  def zero?
    @rows.all? { |row| row.all? { |value| value == 0 } }
  end

  def real?
    @rows.all? { |row| row.all? { |value| value.real? } }
  end

  def symmetric?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    (0...row_size).all? do |row|
      (0...row).all? { |column| self[row, column] == self[column, row] }
    end
  end

  def antisymmetric?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    (0...row_size).all? do |row|
      (0..row).all? { |column| self[row, column] == -self[column, row] }
    end
  end

  def skew_symmetric?
    antisymmetric?
  end

  def diagonal?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    (0...row_size).all? do |row|
      (0...column_size).all? do |column|
        row == column || self[row, column] == 0
      end
    end
  end

  def lower_triangular?
    (0...row_size).all? do |row|
      (0...column_size).all? do |column|
        column <= row || self[row, column] == 0
      end
    end
  end

  def upper_triangular?
    (0...row_size).all? do |row|
      (0...column_size).all? do |column|
        column >= row || self[row, column] == 0
      end
    end
  end

  def permutation?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    return true if row_size == 0
    seen_columns = []
    @rows.all? do |row|
      ones = []
      row.each_with_index do |value, column|
        return false unless value == 0 || value == 1
        ones.push(column) if value == 1
      end
      next false unless ones.size == 1
      next false if seen_columns.include?(ones[0])
      seen_columns.push(ones[0])
      true
    end
  end

  def orthogonal?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    self.t * self == Matrix.identity(row_size)
  end

  def unitary?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    conjugate.t * self == Matrix.identity(row_size)
  end

  def hermitian?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    self == conjugate.t
  end

  def normal?
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    conjugate.t * self == self * conjugate.t
  end

  def regular?
    !singular?
  end

  def singular?
    determinant == 0
  end

  # ── Numbers a matrix stands for ──────────────────────────────────────────

  def trace
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    (0...row_size).inject(0) { |total, index| total + self[index, index] }
  end

  def tr
    trace
  end

  def determinant
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    count = row_size
    return 1 if count == 0
    return self[0, 0] if count == 1
    return self[0, 0] * self[1, 1] - self[0, 1] * self[1, 0] if count == 2
    total = 0
    (0...count).each do |column|
      sign = column.even? ? 1 : -1
      total += sign * self[0, column] * first_minor(0, column).determinant
    end
    total
  end

  def det
    determinant
  end

  def rank
    walked = to_a.map { |row| row.map { |value| value * 1.0 } }
    height = walked.size
    return 0 if height == 0
    width = walked[0].size
    found = 0
    (0...width).each do |column|
      pivot = nil
      (found...height).each do |row|
        pivot = row if pivot.nil? && walked[row][column].abs > 0.000001
      end
      next if pivot.nil?
      swapped = walked[found]
      walked[found] = walked[pivot]
      walked[pivot] = swapped
      lead = walked[found][column]
      (0...width).each { |step| walked[found][step] = walked[found][step] / lead }
      (0...height).each do |row|
        next if row == found
        factor = walked[row][column]
        (0...width).each do |step|
          walked[row][step] = walked[row][step] - factor * walked[found][step]
        end
      end
      found += 1
    end
    found
  end

  def inverse
    raise ExceptionForMatrix::ErrDimensionMismatch unless square?
    reach = determinant
    raise ExceptionForMatrix::ErrNotRegular if reach == 0
    count = row_size
    return Matrix[[1 / reach]] if count == 1
    made = (0...count).map do |row|
      (0...count).map do |column|
        sign = (row + column).even? ? 1 : -1
        Rational(sign * first_minor(column, row).determinant, reach)
      end
    end
    new_matrix(made)
  end

  def inv
    inverse
  end

  def cofactor(row, column)
    sign = (row + column).even? ? 1 : -1
    sign * first_minor(row, column).determinant
  end

  def conjugate
    collect { |value| value.conjugate }
  end

  def conj
    conjugate
  end

  def real
    collect { |value| value.real }
  end

  def imaginary
    collect { |value| value.imaginary }
  end

  def imag
    imaginary
  end

  def rectangular
    [real, imaginary]
  end

  def rect
    rectangular
  end

  def round(digits = 0)
    collect { |value| value.round(digits) }
  end

  # ── Comparing and rendering ──────────────────────────────────────────────

  def ==(other)
    return false unless other.is_a?(Matrix)
    return false unless row_size == other.row_size && column_size == other.column_size
    (0...row_size).all? do |row|
      (0...column_size).all? { |column| self[row, column] == other[row, column] }
    end
  end

  def eql?(other)
    return false unless other.is_a?(Matrix)
    to_a.eql?(other.to_a)
  end

  def hash
    @rows.hash
  end

  def clone
    new_matrix(to_a)
  end

  def coerce(other)
    return [Matrix::Scalar.new(other), self] if other.is_a?(Numeric)
    raise TypeError, "#{self.class} can't be coerced into #{other.class}"
  end

  def inspect
    "#{self.class}[#{@rows.map { |row| row.inspect }.join(", ")}]"
  end

  def to_s
    return "#{self.class}.empty(#{row_size}, #{column_size})" if empty?
    "#{self.class}[#{@rows.map { |row| "[" + row.map { |value| value.to_s }.join(", ") + "]" }.join(", ")}]"
  end

  # A matrix of the same class as this one, holding the given rows.
  def new_matrix(rows, column_count = nil)
    made = self.class.allocate
    made.send(:build_from, rows, column_count)
    made
  end
  private :new_matrix

  # A number standing where a matrix is expected, which is what `coerce`
  # hands back so `2 * matrix` reads.
  class Scalar
    ErrOperationNotDefined = ExceptionForMatrix::ErrOperationNotDefined
    ErrOperationNotImplemented = ExceptionForMatrix::ErrOperationNotImplemented

    def initialize(value)
      @value = value
    end

    def +(other)
      raise ExceptionForMatrix::ErrOperationNotDefined if other.is_a?(Matrix) || other.is_a?(Vector)
      raise TypeError, "wrong argument type" unless other.is_a?(Numeric)
      Scalar.new(@value + other)
    end

    def -(other)
      raise ExceptionForMatrix::ErrOperationNotDefined if other.is_a?(Matrix) || other.is_a?(Vector)
      raise TypeError, "wrong argument type" unless other.is_a?(Numeric)
      Scalar.new(@value - other)
    end

    def *(other)
      return other * @value if other.is_a?(Matrix) || other.is_a?(Vector)
      raise TypeError, "wrong argument type" unless other.is_a?(Numeric)
      Scalar.new(@value * other)
    end

    def /(other)
      return other.inverse * @value if other.is_a?(Matrix)
      raise ExceptionForMatrix::ErrOperationNotDefined if other.is_a?(Vector)
      raise TypeError, "wrong argument type" unless other.is_a?(Numeric)
      Scalar.new(@value / other)
    end

    def **(other)
      raise ExceptionForMatrix::ErrOperationNotImplemented if other.is_a?(Matrix)
      raise ExceptionForMatrix::ErrOperationNotDefined if other.is_a?(Vector)
      raise TypeError, "wrong argument type" unless other.is_a?(Numeric)
      Scalar.new(@value ** other)
    end
  end
end
