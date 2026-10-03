#include "ruby.h"

static VALUE arity_0(VALUE self) { return INT2FIX(0); }
static VALUE arity_1(VALUE self, VALUE a1) { return a1; }
static VALUE arity_2(VALUE self, VALUE a1, VALUE a2) { return a2; }
static VALUE arity_3(VALUE self, VALUE a1, VALUE a2, VALUE a3) { return a3; }
static VALUE arity_4(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4) { return a4; }
static VALUE arity_5(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5) { return a5; }
static VALUE arity_6(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6) { return a6; }
static VALUE arity_7(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7) { return a7; }
static VALUE arity_8(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8) { return a8; }
static VALUE arity_9(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8, VALUE a9) { return a9; }
static VALUE arity_10(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8, VALUE a9, VALUE a10) { return a10; }
static VALUE arity_11(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8, VALUE a9, VALUE a10, VALUE a11) { return a11; }
static VALUE arity_12(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8, VALUE a9, VALUE a10, VALUE a11, VALUE a12) { return a12; }
static VALUE arity_13(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8, VALUE a9, VALUE a10, VALUE a11, VALUE a12, VALUE a13) { return a13; }
static VALUE arity_14(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8, VALUE a9, VALUE a10, VALUE a11, VALUE a12, VALUE a13, VALUE a14) { return a14; }
static VALUE arity_15(VALUE self, VALUE a1, VALUE a2, VALUE a3, VALUE a4, VALUE a5, VALUE a6, VALUE a7, VALUE a8, VALUE a9, VALUE a10, VALUE a11, VALUE a12, VALUE a13, VALUE a14, VALUE a15) { return a15; }

static VALUE counted(int argc, VALUE *argv, VALUE self) { return INT2FIX(argc); }
static VALUE gathered(VALUE self, VALUE arguments) { return arguments; }
static VALUE same(VALUE self, VALUE first, VALUE second) { return first == second ? Qtrue : Qfalse; }
static VALUE class_of(VALUE self, VALUE object) { return rb_class_of(object); }
static VALUE unknown_handle(VALUE self) { return (VALUE)0x7ff00000; }
static VALUE reopen(VALUE self) { return rb_define_class("CMethods", rb_cObject); }
static VALUE mismatch(VALUE self) { return rb_define_class("CMethods", rb_class_of(self)); }
static VALUE integer_superclass(VALUE self) { return rb_define_class("Elsewhere", INT2FIX(1)); }
static VALUE constant_taken(VALUE self) { return rb_define_class("RUBY_VERSION", rb_cObject); }
static VALUE child_of(VALUE self, VALUE superclass) { return rb_define_class("CChild", superclass); }

static VALUE arity_too_large(VALUE self) {
  rb_define_method(rb_class_of(self), "too_many", arity_0, 16);
  return Qnil;
}

static VALUE method_on_integer(VALUE self) {
  rb_define_method(INT2FIX(1), "anything", arity_0, 0);
  return Qnil;
}

void Init_c_methods(void) {
  VALUE cls = rb_define_class("CMethods", rb_cObject);
  rb_define_method(cls, "arity_0", arity_0, 0);
  rb_define_method(cls, "arity_1", arity_1, 1);
  rb_define_method(cls, "arity_2", arity_2, 2);
  rb_define_method(cls, "arity_3", arity_3, 3);
  rb_define_method(cls, "arity_4", arity_4, 4);
  rb_define_method(cls, "arity_5", arity_5, 5);
  rb_define_method(cls, "arity_6", arity_6, 6);
  rb_define_method(cls, "arity_7", arity_7, 7);
  rb_define_method(cls, "arity_8", arity_8, 8);
  rb_define_method(cls, "arity_9", arity_9, 9);
  rb_define_method(cls, "arity_10", arity_10, 10);
  rb_define_method(cls, "arity_11", arity_11, 11);
  rb_define_method(cls, "arity_12", arity_12, 12);
  rb_define_method(cls, "arity_13", arity_13, 13);
  rb_define_method(cls, "arity_14", arity_14, 14);
  rb_define_method(cls, "arity_15", arity_15, 15);
  rb_define_method(cls, "counted", counted, -1);
  rb_define_method(cls, "gathered", gathered, -2);
  rb_define_method(cls, "same", same, 2);
  rb_define_method(cls, "class_of", class_of, 1);
  rb_define_method(cls, "unknown_handle", unknown_handle, 0);
  rb_define_method(cls, "reopen", reopen, 0);
  rb_define_method(cls, "mismatch", mismatch, 0);
  rb_define_method(cls, "integer_superclass", integer_superclass, 0);
  rb_define_method(cls, "constant_taken", constant_taken, 0);
  rb_define_method(cls, "child_of", child_of, 1);
  rb_define_method(cls, "arity_too_large", arity_too_large, 0);
  rb_define_method(cls, "method_on_integer", method_on_integer, 0);
}
