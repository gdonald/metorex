#include "ruby.h"

static VALUE compare_error(VALUE self, VALUE first, VALUE second) { rb_cmperr(first, second); }
static VALUE divide_error(VALUE self) { rb_num_zerodiv(); }
static VALUE sign(VALUE self, VALUE answer) { return INT2FIX(rb_cmpint(answer, INT2FIX(1), INT2FIX(2))); }
static VALUE binary(VALUE self, VALUE first, VALUE second) { return rb_num_coerce_bin(first, second, rb_intern("+")); }
static VALUE compared(VALUE self, VALUE first, VALUE second) { return rb_num_coerce_cmp(first, second, rb_intern("<=>")); }
static VALUE related(VALUE self, VALUE first, VALUE second) { return rb_num_coerce_relop(first, second, rb_intern("<")); }
static VALUE integer(VALUE self, VALUE value) { return rb_Integer(value); }
static VALUE short_of(VALUE self, VALUE value) { return INT2FIX(NUM2SHORT(value)); }
static VALUE character(VALUE self, VALUE value) { return CHR2FIX(NUM2CHR(value)); }
static VALUE single_bit(VALUE self, VALUE value) { return INT2FIX(rb_absint_singlebit_p(value)); }
static VALUE long_long_size(VALUE self) { return INT2FIX(sizeof(LONG_LONG)); }

void Init_c_coercion(void) {
  VALUE cls = rb_define_class("CCoercion", rb_cObject);
  rb_define_method(cls, "compare_error", compare_error, 2);
  rb_define_method(cls, "divide_error", divide_error, 0);
  rb_define_method(cls, "sign", sign, 1);
  rb_define_method(cls, "binary", binary, 2);
  rb_define_method(cls, "compared", compared, 2);
  rb_define_method(cls, "related", related, 2);
  rb_define_method(cls, "integer", integer, 1);
  rb_define_method(cls, "short_of", short_of, 1);
  rb_define_method(cls, "character", character, 1);
  rb_define_method(cls, "single_bit", single_bit, 1);
  rb_define_method(cls, "long_long_size", long_long_size, 0);
}
