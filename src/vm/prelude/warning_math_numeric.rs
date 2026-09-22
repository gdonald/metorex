pub(super) const SOURCE: &str = r##"
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
  # Exact division. A number metorex has no arithmetic of its own for is
  # asked for the rational it stands for, and that is divided instead.
  def quo(other)
    converted = to_r
    unless converted.is_a?(Rational)
      raise TypeError, "can't convert #{self.class} into Rational"
    end
    converted / other
  end

  # A real number stands at angle zero when it is positive and at Pi when it
  # is negative, which is where it sits on the complex plane.
  def arg
    return self if respond_to?(:nan?) && nan?
    self < 0 ? Math::PI : 0
  end

  alias_method :angle, :arg
  alias_method :phase, :arg

  # The same number written as a distance and an angle.
  def polar
    [abs, arg]
  end

  # A real number has no imaginary part, so its rectangular form is itself
  # and zero.
  def rect
    [self, 0]
  end

  alias_method :rectangular, :rect

  # The square of the distance from zero, which a number reaches by
  # multiplying itself.
  def abs2
    self * self
  end

  # `2.i` is the complex number whose imaginary part is the number itself.
  def i
    Complex(0, self)
  end

  # A number stands as the real part of a complex number with no imaginary
  # part of its own.
  def to_c
    Complex(self, 0)
  end

  # Ruby negates a number it has no arithmetic of its own for by asking it to
  # coerce zero, then subtracting it from what came back.
  def -@
    first, second = coerce(0)
    first - second
  end

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
  # differ by one divisor whenever the signs disagree. Anything that is not
  # a number is asked to coerce the pair first.
  def remainder(other)
    mine = self
    mine, other = other.coerce(self) unless other.is_a?(Numeric)
    left = mine % other
    return left if left == 0
    return left - other if (mine < 0 && other > 0) || (mine > 0 && other < 0)
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

# Enumerable, written over `each` the way Ruby writes it. A class that
# defines `each` and includes this answers the whole family. Array, Hash,
# Range, and Struct reach their own native implementations first."##;
