#include "ruby.h"

static VALUE define(VALUE self, VALUE name) {
  return rb_struct_define(NIL_P(name) ? NULL : RSTRING_PTR(name), "left", "right", NULL);
}
static VALUE define_twice(VALUE self) { return rb_struct_define(NULL, "same", "same", NULL); }
static VALUE define_under(VALUE self, VALUE outer, VALUE name) {
  return rb_struct_define_under(outer, RSTRING_PTR(name), "first", NULL);
}
static VALUE define_data(VALUE self, VALUE superclass) {
  return rb_data_define(NIL_P(superclass) ? 0 : superclass, "width", "height", NULL);
}
static VALUE define_data_twice(VALUE self) { return rb_data_define(0, "width", "width", NULL); }
static VALUE make(VALUE self, VALUE klass, VALUE left, VALUE right) { return rb_struct_new(klass, left, right); }
static VALUE make_with(VALUE self, VALUE klass, VALUE values) {
  return rb_class_new_instance((int)RARRAY_LEN(values), RARRAY_PTR(values), klass);
}
static VALUE class_members(VALUE self, VALUE klass) { return rb_ary_dup(rb_struct_s_members(klass)); }
static VALUE members(VALUE self, VALUE instance) { return rb_struct_members(instance); }
static VALUE size(VALUE self, VALUE instance) { return rb_struct_size(instance); }
static VALUE read_member(VALUE self, VALUE instance, VALUE key) { return rb_struct_aref(instance, key); }
static VALUE write_member(VALUE self, VALUE instance, VALUE key, VALUE value) { return rb_struct_aset(instance, key, value); }
static VALUE member(VALUE self, VALUE instance, VALUE name) { return rb_struct_getmember(instance, SYM2ID(name)); }
static VALUE fill(VALUE self, VALUE instance, VALUE values) { return rb_struct_initialize(instance, values); }

void Init_c_structs(void) {
  VALUE cls = rb_define_class("CStructs", rb_cObject);
  rb_define_method(cls, "define", define, 1);
  rb_define_method(cls, "define_twice", define_twice, 0);
  rb_define_method(cls, "define_under", define_under, 2);
  rb_define_method(cls, "define_data", define_data, 1);
  rb_define_method(cls, "define_data_twice", define_data_twice, 0);
  rb_define_method(cls, "make", make, 3);
  rb_define_method(cls, "make_with", make_with, 2);
  rb_define_method(cls, "class_members", class_members, 1);
  rb_define_method(cls, "members", members, 1);
  rb_define_method(cls, "size", size, 1);
  rb_define_method(cls, "read", read_member, 2);
  rb_define_method(cls, "write", write_member, 3);
  rb_define_method(cls, "member", member, 2);
  rb_define_method(cls, "fill", fill, 2);
}
