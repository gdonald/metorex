#include "ruby.h"

static VALUE bytes(VALUE self, VALUE string) {
  const char *pointer = RSTRING_PTR(string);
  long length = RSTRING_LEN(string);
  VALUE answered = rb_funcall(rb_cObject, rb_intern("const_get"), 1, ID2SYM(rb_intern("Array")));
  answered = rb_funcall(answered, rb_intern("new"), 0);
  for (long index = 0; index < length; index++) {
    rb_funcall(answered, rb_intern("push"), 1, INT2FIX((unsigned char)pointer[index]));
  }
  return answered;
}
static VALUE terminated(VALUE self, VALUE string) {
  return RSTRING_PTR(string)[RSTRING_LEN(string)] == '\0' ? Qtrue : Qfalse;
}
static VALUE same_pointer(VALUE self, VALUE string) {
  return RSTRING_PTR(string) == RSTRING_PTR(string) ? Qtrue : Qfalse;
}
static VALUE path_value(VALUE self, VALUE object) { return FilePathValue(object); }
static VALUE open_name(VALUE self, VALUE name, VALUE mode) {
  return rb_file_open(RSTRING_PTR(name), RSTRING_PTR(mode));
}
static VALUE open_path(VALUE self, VALUE name, VALUE mode) {
  return rb_file_open_str(name, RSTRING_PTR(mode));
}

void Init_c_files(void) {
  VALUE cls = rb_define_class("CFiles", rb_cObject);
  rb_define_method(cls, "bytes", bytes, 1);
  rb_define_method(cls, "terminated", terminated, 1);
  rb_define_method(cls, "same_pointer", same_pointer, 1);
  rb_define_method(cls, "path_value", path_value, 1);
  rb_define_method(cls, "open_name", open_name, 2);
  rb_define_method(cls, "open_path", open_path, 2);
}
