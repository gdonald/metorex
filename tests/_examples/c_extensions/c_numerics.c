#include "ruby.h"

static VALUE half(VALUE self) { return DBL2NUM(0.5); }
static VALUE doubled(VALUE self, VALUE number) { return rb_float_new(RFLOAT_VALUE(number) * 2); }
static VALUE is_float(VALUE self, VALUE value) { return RB_FLOAT_TYPE_P(value) ? Qtrue : Qfalse; }
static VALUE to_float(VALUE self, VALUE value) { return rb_Float(value); }
static VALUE complex(VALUE self, VALUE real, VALUE imaginary) { return rb_Complex(real, imaginary); }
static VALUE complex_real(VALUE self, VALUE real) { return rb_Complex1(real); }
static VALUE complex_new(VALUE self, VALUE real, VALUE imaginary) { return rb_complex_new2(real, imaginary); }
static VALUE complex_new_real(VALUE self, VALUE real) { return rb_complex_new1(real); }
static VALUE rational(VALUE self, VALUE numerator, VALUE denominator) { return rb_Rational2(numerator, denominator); }
static VALUE rational_whole(VALUE self, VALUE numerator) { return rb_Rational1(numerator); }
static VALUE rational_new(VALUE self, VALUE numerator, VALUE denominator) {
  return rb_rational_new2(numerator, denominator);
}
static VALUE rational_new_whole(VALUE self, VALUE numerator) { return rb_rational_new1(numerator); }
static VALUE parts(VALUE self, VALUE rational) {
  VALUE values[2] = {rb_rational_num(rational), rb_rational_den(rational)};
  return rb_ary_new_from_values(2, values);
}

void Init_c_numerics(void) {
  VALUE cls = rb_define_class("CNumerics", rb_cObject);
  rb_define_method(cls, "half", half, 0);
  rb_define_method(cls, "doubled", doubled, 1);
  rb_define_method(cls, "is_float", is_float, 1);
  rb_define_method(cls, "to_float", to_float, 1);
  rb_define_method(cls, "complex", complex, 2);
  rb_define_method(cls, "complex_real", complex_real, 1);
  rb_define_method(cls, "complex_new", complex_new, 2);
  rb_define_method(cls, "complex_new_real", complex_new_real, 1);
  rb_define_method(cls, "rational", rational, 2);
  rb_define_method(cls, "rational_whole", rational_whole, 1);
  rb_define_method(cls, "rational_new", rational_new, 2);
  rb_define_method(cls, "rational_new_whole", rational_new_whole, 1);
  rb_define_method(cls, "parts", parts, 1);
}
