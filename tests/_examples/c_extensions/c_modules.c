#include "ruby.h"

static VALUE answer(VALUE self) { return ID2SYM(rb_intern("answered")); }
static VALUE one(VALUE self, VALUE first) { return first; }
static VALUE any(int count, VALUE *values, VALUE self) { return INT2FIX(count); }
static VALUE listed(VALUE self, VALUE values) { return values; }

static VALUE defined(VALUE self, VALUE module, VALUE name) {
  return rb_ary_new_from_args(2, rb_const_defined(module, SYM2ID(name)) ? Qtrue : Qfalse,
                              rb_const_defined_at(module, SYM2ID(name)) ? Qtrue : Qfalse);
}
static VALUE get(VALUE self, VALUE module, VALUE name) { return rb_const_get(module, SYM2ID(name)); }
static VALUE get_at(VALUE self, VALUE module, VALUE name) { return rb_const_get_at(module, SYM2ID(name)); }
static VALUE get_from(VALUE self, VALUE module, VALUE name) { return rb_const_get_from(module, SYM2ID(name)); }
static VALUE set(VALUE self, VALUE module, VALUE name, VALUE value) {
  rb_const_set(module, SYM2ID(name), value);
  return Qnil;
}
static VALUE global_const(VALUE self, VALUE name, VALUE value) {
  rb_define_global_const(RSTRING_PTR(name), value);
  return Qnil;
}
static VALUE aliases(VALUE self, VALUE module) {
  rb_define_alias(module, "first_alias", "original");
  rb_alias(module, rb_intern("second_alias"), rb_intern("original"));
  return Qnil;
}
static VALUE methods(VALUE self, VALUE module) {
  rb_define_method(module, "plain", answer, 0);
  rb_define_method(module, "one", one, 1);
  rb_define_method(module, "any", any, -1);
  rb_define_method(module, "listed", listed, -2);
  rb_define_private_method(module, "hidden", answer, 0);
  rb_define_protected_method(module, "guarded", answer, 0);
  return Qnil;
}
static VALUE singleton(VALUE self, VALUE object) {
  rb_define_singleton_method(object, "only_here", answer, 0);
  return Qnil;
}
static VALUE module_function(VALUE self, VALUE module) {
  rb_define_module_function(module, "both_ways", answer, 0);
  return Qnil;
}
static VALUE global_function(VALUE self) {
  rb_define_global_function("c_modules_everywhere", answer, 0);
  return Qnil;
}
static VALUE undefine(VALUE self, VALUE module, VALUE name) {
  rb_undef_method(module, RSTRING_PTR(name));
  return Qnil;
}
static VALUE undefine_strictly(VALUE self, VALUE module, VALUE name) {
  rb_undef(module, SYM2ID(name));
  return Qnil;
}
static VALUE naming(VALUE self, VALUE module) {
  return rb_ary_new_from_args(3, rb_str_new_cstr(rb_class2name(module)), rb_mod_name(module), rb_class_name(module));
}
static VALUE ancestors(VALUE self, VALUE module) { return rb_mod_ancestors(module); }

void Init_c_modules(void) {
  VALUE cls = rb_define_class("CModules", rb_cObject);
  rb_define_method(cls, "defined", defined, 2);
  rb_define_method(cls, "get", get, 2);
  rb_define_method(cls, "get_at", get_at, 2);
  rb_define_method(cls, "get_from", get_from, 2);
  rb_define_method(cls, "set", set, 3);
  rb_define_method(cls, "global_const", global_const, 2);
  rb_define_method(cls, "aliases", aliases, 1);
  rb_define_method(cls, "methods_on", methods, 1);
  rb_define_method(cls, "singleton", singleton, 1);
  rb_define_method(cls, "module_function", module_function, 1);
  rb_define_method(cls, "global_function", global_function, 0);
  rb_define_method(cls, "undefine", undefine, 2);
  rb_define_method(cls, "undefine_strictly", undefine_strictly, 2);
  rb_define_method(cls, "naming", naming, 1);
  rb_define_method(cls, "ancestors", ancestors, 1);
}
