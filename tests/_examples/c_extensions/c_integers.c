#include "ruby.h"

static VALUE write_byte(VALUE self, VALUE string, VALUE index, VALUE byte) {
  RSTRING_PTR(string)[FIX2LONG(index)] = (char)FIX2LONG(byte);
  return string;
}
static VALUE write_then_copy(VALUE self, VALUE string, VALUE byte) {
  RSTRING_PTR(string)[0] = (char)FIX2LONG(byte);
  return rb_funcall(string, rb_intern("dup"), 0);
}
static VALUE pointer_kept(VALUE self, VALUE string, VALUE change) {
  char *before = RSTRING_PTR(string);
  rb_funcall(string, SYM2ID(change), 0);
  return before == RSTRING_PTR(string) ? Qtrue : Qfalse;
}
static VALUE read_after(VALUE self, VALUE string, VALUE change) {
  RSTRING_PTR(string);
  rb_funcall(string, SYM2ID(change), 0);
  VALUE values[1] = {string};
  const char *pointer = RSTRING_PTR(string);
  long length = RSTRING_LEN(string);
  VALUE bytes = rb_ary_new_from_values(0, values);
  for (long index = 0; index < length; index++) {
    rb_funcall(bytes, rb_intern("push"), 1, INT2FIX((unsigned char)pointer[index]));
  }
  return bytes;
}
static VALUE pack(VALUE self, VALUE value, VALUE words, VALUE count, VALUE size, VALUE nails,
                  VALUE flags) {
  return INT2FIX(rb_integer_pack(value, RSTRING_PTR(words), FIX2LONG(count), FIX2LONG(size),
                                 FIX2LONG(nails), FIX2INT(flags)));
}
RUBY_EXTERN VALUE rb_int_positive_pow(long base, unsigned long exponent);
static VALUE power(VALUE self, VALUE base, VALUE exponent) {
  return rb_int_positive_pow(FIX2LONG(base), FIX2LONG(exponent));
}
static VALUE define_const(VALUE self, VALUE module, VALUE name, VALUE value) {
  rb_define_const(module, RSTRING_PTR(name), value);
  return Qnil;
}

void Init_c_integers(void) {
  VALUE cls = rb_define_class("CIntegers", rb_cObject);
  rb_define_const(cls, "BIG_ENDIAN", INT2NUM(INTEGER_PACK_BIG_ENDIAN));
  rb_define_const(cls, "LITTLE_ENDIAN", INT2NUM(INTEGER_PACK_LITTLE_ENDIAN));
  rb_define_const(cls, "MSWORD", INT2NUM(INTEGER_PACK_MSWORD_FIRST));
  rb_define_const(cls, "LSWORD", INT2NUM(INTEGER_PACK_LSWORD_FIRST));
  rb_define_const(cls, "MSBYTE", INT2NUM(INTEGER_PACK_MSBYTE_FIRST));
  rb_define_const(cls, "LSBYTE", INT2NUM(INTEGER_PACK_LSBYTE_FIRST));
  rb_define_const(cls, "NATIVE", INT2NUM(INTEGER_PACK_NATIVE_BYTE_ORDER));
  rb_define_const(cls, "PACK_2COMP", INT2NUM(INTEGER_PACK_2COMP));
  rb_define_const(cls, "FORCE_BIGNUM", INT2NUM(INTEGER_PACK_FORCE_BIGNUM));
  rb_define_const(cls, "GENERIC", INT2NUM(INTEGER_PACK_FORCE_GENERIC_IMPLEMENTATION));
  rb_define_method(cls, "write_byte", write_byte, 3);
  rb_define_method(cls, "write_then_copy", write_then_copy, 2);
  rb_define_method(cls, "pointer_kept", pointer_kept, 2);
  rb_define_method(cls, "read_after", read_after, 2);
  rb_define_method(cls, "pack", pack, 6);
  rb_define_method(cls, "power", power, 2);
  rb_define_method(cls, "define_const", define_const, 3);
}
