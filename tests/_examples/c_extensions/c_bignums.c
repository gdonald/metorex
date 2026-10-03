#include "ruby.h"

static VALUE to_long(VALUE self, VALUE value) { return LONG2NUM(rb_big2long(value)); }
static VALUE to_long_long(VALUE self, VALUE value) { return LL2NUM(rb_big2ll(value)); }
static VALUE to_unsigned_long(VALUE self, VALUE value) { return ULONG2NUM(rb_big2ulong(value)); }
static VALUE to_double(VALUE self, VALUE value) { return DBL2NUM(rb_big2dbl(value)); }
static VALUE from_double(VALUE self, VALUE value) { return rb_dbl2big(RFLOAT_VALUE(value)); }
static VALUE to_text(VALUE self, VALUE value, VALUE base) { return rb_big2str(value, FIX2INT(base)); }
static VALUE sign(VALUE self, VALUE value) {
  VALUE values[3] = {INT2FIX(RBIGNUM_SIGN(value)), RBIGNUM_POSITIVE_P(value) ? Qtrue : Qfalse,
                     RBIGNUM_NEGATIVE_P(value) ? Qtrue : Qfalse};
  return rb_ary_new_from_values(3, values);
}
static VALUE compare(VALUE self, VALUE first, VALUE second) { return rb_big_cmp(first, second); }
static VALUE pack(VALUE self, VALUE value, VALUE count) {
  long longs = FIX2LONG(count);
  unsigned long held[4] = {1, 1, 1, 1};
  rb_big_pack(value, held, longs);
  VALUE packed = rb_ary_new_capa(longs);
  for (long index = 0; index < longs; index++) rb_ary_push(packed, ULONG2NUM(held[index]));
  return packed;
}
static VALUE size(VALUE self, VALUE value) {
  int unused = -1;
  size_t bytes = rb_absint_size(value, &unused);
  VALUE values[3] = {SIZET2NUM(bytes), INT2FIX(unused), SIZET2NUM(rb_absint_size(value, NULL))};
  return rb_ary_new_from_values(3, values);
}
static VALUE to_c_double(VALUE self, VALUE value) { return DBL2NUM(NUM2DBL(value)); }
static VALUE empty(VALUE self) { return rb_ary_new2(10); }

void Init_c_bignums(void) {
  VALUE cls = rb_define_class("CBignums", rb_cObject);
  rb_define_method(cls, "to_long", to_long, 1);
  rb_define_method(cls, "to_long_long", to_long_long, 1);
  rb_define_method(cls, "to_unsigned_long", to_unsigned_long, 1);
  rb_define_method(cls, "to_double", to_double, 1);
  rb_define_method(cls, "from_double", from_double, 1);
  rb_define_method(cls, "to_text", to_text, 2);
  rb_define_method(cls, "sign", sign, 1);
  rb_define_method(cls, "compare", compare, 2);
  rb_define_method(cls, "pack", pack, 2);
  rb_define_method(cls, "size", size, 1);
  rb_define_method(cls, "to_c_double", to_c_double, 1);
  rb_define_method(cls, "empty", empty, 0);
}
