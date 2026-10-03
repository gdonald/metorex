#include "ruby.h"

static VALUE call_super(VALUE self) { return rb_call_super(0, 0); }
static VALUE call_super_with(VALUE self, VALUE first) { return rb_call_super(1, &first); }

static VALUE super_calling(VALUE self, VALUE klass, VALUE name) {
  rb_define_method(klass, RSTRING_PTR(name), call_super, 0);
  return Qnil;
}
static VALUE super_calling_with(VALUE self, VALUE klass, VALUE name) {
  rb_define_method(klass, RSTRING_PTR(name), call_super_with, 1);
  return Qnil;
}
static VALUE path(VALUE self, VALUE klass) { return rb_class_path(klass); }
static VALUE path2class(VALUE self, VALUE path) { return rb_path2class(RSTRING_PTR(path)); }
static VALUE path_to_class(VALUE self, VALUE path) { return rb_path_to_class(path); }
static VALUE instance_methods(VALUE self, VALUE klass, VALUE inherited) {
  return rb_ary_new_from_args(4, rb_class_instance_methods(1, &inherited, klass),
                              rb_class_public_instance_methods(1, &inherited, klass),
                              rb_class_protected_instance_methods(1, &inherited, klass),
                              rb_class_private_instance_methods(1, &inherited, klass));
}
static VALUE class_new(VALUE self, VALUE superclass) { return rb_class_new(superclass); }
static VALUE new_instance_kw(VALUE self, VALUE arguments, VALUE klass) {
  return rb_class_new_instance_kw(RARRAY_LENINT(arguments), RARRAY_PTR(arguments), klass, RB_PASS_KEYWORDS);
}
static VALUE real(VALUE self, VALUE object) { return rb_class_real(CLASS_OF(object)); }
static VALUE real_of_null(VALUE self) { return rb_class_real(0) == 0 ? Qtrue : Qfalse; }
static VALUE superclasses(VALUE self, VALUE klass) {
  return rb_ary_new_from_args(2, rb_class_superclass(klass), rb_class_get_superclass(klass));
}
static VALUE cvar_defined(VALUE self, VALUE klass, VALUE name) {
  return rb_cvar_defined(klass, rb_intern(RSTRING_PTR(name)));
}
static VALUE cvars(VALUE self, VALUE klass) {
  rb_cvar_set(klass, rb_intern("@@by_id"), INT2FIX(1));
  rb_cv_set(klass, "@@by_name", INT2FIX(2));
  rb_define_class_variable(klass, "@@defined", INT2FIX(3));
  return rb_ary_new_from_args(3, rb_cvar_get(klass, rb_intern("@@by_id")), rb_cv_get(klass, "@@by_name"),
                              rb_cv_get(klass, "@@defined"));
}
static VALUE cv_get(VALUE self, VALUE klass, VALUE name) { return rb_cv_get(klass, RSTRING_PTR(name)); }
static VALUE attrs(VALUE self, VALUE klass) {
  rb_define_attr(klass, "readable", 1, 0);
  rb_define_attr(klass, "writable", 0, 1);
  rb_define_attr(klass, "both", 1, 1);
  return Qnil;
}
static VALUE include_module(VALUE self, VALUE klass, VALUE module) {
  rb_include_module(klass, module);
  return klass;
}
static VALUE type_of(VALUE self, VALUE object) { return INT2FIX(TYPE(object)); }
static VALUE is_type(VALUE self, VALUE object, VALUE type) {
  return RB_TYPE_P(object, FIX2INT(type)) ? Qtrue : Qfalse;
}
static VALUE define_class(VALUE self, VALUE outer, VALUE name, VALUE superclass) {
  if (NIL_P(superclass)) superclass = 0;
  return rb_define_class_under(outer, RSTRING_PTR(name), superclass);
}

void Init_c_classes(void) {
  VALUE klass = rb_define_class("CClasses", rb_cObject);
  rb_define_method(klass, "super_calling", super_calling, 2);
  rb_define_method(klass, "super_calling_with", super_calling_with, 2);
  rb_define_method(klass, "path", path, 1);
  rb_define_method(klass, "path2class", path2class, 1);
  rb_define_method(klass, "path_to_class", path_to_class, 1);
  rb_define_method(klass, "instance_methods_of", instance_methods, 2);
  rb_define_method(klass, "class_new", class_new, 1);
  rb_define_method(klass, "new_instance_kw", new_instance_kw, 2);
  rb_define_method(klass, "real", real, 1);
  rb_define_method(klass, "real_of_null", real_of_null, 0);
  rb_define_method(klass, "superclasses", superclasses, 1);
  rb_define_method(klass, "cvar_defined", cvar_defined, 2);
  rb_define_method(klass, "cvars", cvars, 1);
  rb_define_method(klass, "cv_get", cv_get, 2);
  rb_define_method(klass, "attrs", attrs, 1);
  rb_define_method(klass, "include_module", include_module, 2);
  rb_define_method(klass, "type_of", type_of, 1);
  rb_define_method(klass, "is_type", is_type, 2);
  rb_define_method(klass, "define_class", define_class, 3);
  rb_define_const(klass, "T_FIXNUM", INT2FIX(T_FIXNUM));
  rb_define_const(klass, "T_STRING", INT2FIX(T_STRING));
}
