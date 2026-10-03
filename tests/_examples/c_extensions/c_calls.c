#include "ruby.h"

static VALUE symbol_for(VALUE self, VALUE name) { return ID2SYM(rb_intern(rb_id2name(SYM2ID(name)))); }
static VALUE same_id(VALUE self, VALUE first, VALUE second) { return SYM2ID(first) == SYM2ID(second) ? Qtrue : Qfalse; }
static VALUE call_none(VALUE self, VALUE receiver, VALUE name) { return rb_funcall(receiver, SYM2ID(name), 0); }
static VALUE call_two(VALUE self, VALUE receiver, VALUE name, VALUE first, VALUE second) {
  return rb_funcall(receiver, SYM2ID(name), 2, first, second);
}
static VALUE constant(VALUE self, VALUE module, VALUE name) { return rb_const_get(module, SYM2ID(name)); }
static VALUE dump(VALUE self, VALUE object, VALUE port) { return rb_marshal_dump(object, port); }
static VALUE load(VALUE self, VALUE data) { return rb_marshal_load(data); }
static VALUE define_finalizer(VALUE self, VALUE object, VALUE finalizer) { return rb_define_finalizer(object, finalizer); }
static VALUE undefine_finalizer(VALUE self, VALUE object) { return rb_undefine_finalizer(object); }

static VALUE to_long(VALUE self, VALUE value) { return LONG2FIX(NUM2LONG(value)); }
/* Quartered, since a whole unsigned long does not fit a fixnum. */
static VALUE quarter_unsigned_long(VALUE self, VALUE value) { return LONG2FIX((long)(NUM2ULONG(value) >> 2)); }
static VALUE to_int(VALUE self, VALUE value) { return INT2NUM(NUM2INT(value)); }
static VALUE to_unsigned_int(VALUE self, VALUE value) { return UINT2NUM(NUM2UINT(value)); }
static VALUE fix_to_int(VALUE self, VALUE value) { return INT2NUM(FIX2INT(value)); }
static VALUE fix_to_unsigned_int(VALUE self, VALUE value) { return UINT2NUM(FIX2UINT(value)); }

static VALUE class_under(VALUE self, VALUE outer, VALUE name, VALUE superclass) {
  return rb_define_class_under(outer, rb_id2name(SYM2ID(name)), superclass);
}
static VALUE class_id_under(VALUE self, VALUE outer, VALUE name, VALUE superclass) {
  return rb_define_class_id_under(outer, SYM2ID(name), superclass);
}
static VALUE module(VALUE self, VALUE name) { return rb_define_module(rb_id2name(SYM2ID(name))); }
static VALUE module_under(VALUE self, VALUE outer, VALUE name) {
  return rb_define_module_under(outer, rb_id2name(SYM2ID(name)));
}

void Init_c_calls(void) {
  VALUE cls = rb_define_class("CCalls", rb_cObject);
  rb_define_method(cls, "symbol_for", symbol_for, 1);
  rb_define_method(cls, "same_id", same_id, 2);
  rb_define_method(cls, "call_none", call_none, 2);
  rb_define_method(cls, "call_two", call_two, 4);
  rb_define_method(cls, "constant", constant, 2);
  rb_define_method(cls, "dump", dump, 2);
  rb_define_method(cls, "load", load, 1);
  rb_define_method(cls, "define_finalizer", define_finalizer, 2);
  rb_define_method(cls, "undefine_finalizer", undefine_finalizer, 1);
  rb_define_method(cls, "to_long", to_long, 1);
  rb_define_method(cls, "quarter_unsigned_long", quarter_unsigned_long, 1);
  rb_define_method(cls, "to_int", to_int, 1);
  rb_define_method(cls, "to_unsigned_int", to_unsigned_int, 1);
  rb_define_method(cls, "fix_to_int", fix_to_int, 1);
  rb_define_method(cls, "fix_to_unsigned_int", fix_to_unsigned_int, 1);
  rb_define_method(cls, "class_under", class_under, 3);
  rb_define_method(cls, "class_id_under", class_id_under, 3);
  rb_define_method(cls, "module", module, 1);
  rb_define_method(cls, "module_under", module_under, 2);
}
