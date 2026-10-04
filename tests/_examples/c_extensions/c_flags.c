#include "ruby.h"

static VALUE flags(VALUE self, VALUE object) { return ULONG2NUM(RBASIC(object)->flags & (FL_FREEZE | FL_USER0)); }
static VALUE set_flags(VALUE self, VALUE object, VALUE value) {
  RBASIC(object)->flags = NUM2ULONG(value);
  return object;
}
static VALUE freeze_flag(VALUE self) { return ULONG2NUM(FL_FREEZE); }
static VALUE user_flag(VALUE self) { return ULONG2NUM(FL_USER0); }
static VALUE klass(VALUE self, VALUE object) { return RBASIC(object)->klass; }
static VALUE test_flag(VALUE self, VALUE object) { return ULONG2NUM(FL_TEST(object, FL_FREEZE)); }
static VALUE set_user_flag(VALUE self, VALUE object) {
  FL_SET(object, FL_USER0);
  return object;
}
static VALUE unset_user_flag(VALUE self, VALUE object) {
  FL_UNSET(object, FL_USER0);
  return object;
}
static VALUE special_const(VALUE self, VALUE object) { return SPECIAL_CONST_P(object) ? Qtrue : Qfalse; }

void Init_c_flags(void) {
  VALUE cls = rb_define_class("CFlags", rb_cObject);
  rb_define_method(cls, "flags", flags, 1);
  rb_define_method(cls, "set_flags", set_flags, 2);
  rb_define_method(cls, "freeze_flag", freeze_flag, 0);
  rb_define_method(cls, "user_flag", user_flag, 0);
  rb_define_method(cls, "klass", klass, 1);
  rb_define_method(cls, "test_flag", test_flag, 1);
  rb_define_method(cls, "set_user_flag", set_user_flag, 1);
  rb_define_method(cls, "unset_user_flag", unset_user_flag, 1);
  rb_define_method(cls, "special_const", special_const, 1);
}
