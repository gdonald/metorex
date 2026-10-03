#include "ruby.h"

static VALUE numbers(VALUE self) {
  return rb_sprintf("%d|%5d|%-4d|%05d|%ld|%lld|%u|%x|%X|%o|%zu|%hd|%c|%%", -7, 42, 3, 9, 123456789012L,
                    -5LL, 4000000000U, 255, 255, 8, (size_t)17, (short)-2, 'z');
}
static VALUE floats(VALUE self) {
  return rb_sprintf("%.2f|%e|%g|%*.*f|%Lf", 3.14159, 1500.0, 0.0001, 8, 3, 2.5, (long double)1.25);
}
static VALUE texts(VALUE self, VALUE value) {
  return rb_sprintf("%s|%.3s|%-6s|[%" PRIsVALUE "]|[%+" PRIsVALUE "]", "plain", "abcdef", "ab", value, value);
}
static VALUE pointer_text(VALUE self) {
  VALUE written = rb_sprintf("%p", (void *)0);
  return RSTRING_LEN(written) > 0 ? Qtrue : Qfalse;
}
static VALUE unknown(VALUE self) { return rb_sprintf("%y and %", 1); }
static VALUE appended(VALUE self, VALUE string) { return rb_str_catf(string, " and %d more", 2); }
static VALUE raised(VALUE self, VALUE klass, VALUE value) {
  rb_raise(klass, "bad %" PRIsVALUE " at %d", value, 7);
}
static VALUE warned(VALUE self, VALUE value) {
  rb_warn("careful with %" PRIsVALUE, value);
  rb_warning("only when verbose: %d", 1);
  return Qnil;
}
static VALUE bytes(VALUE self) {
  VALUE values[3] = {rb_str_new("a\0b", 3), rb_str_new2("c string"), rb_str_new(NULL, 0)};
  return rb_ary_new_from_values(3, values);
}
static VALUE as_strings(VALUE self, VALUE value) {
  VALUE values[2] = {rb_obj_as_string(value), rb_inspect(value)};
  return rb_ary_new_from_values(2, values);
}

void Init_c_formats(void) {
  VALUE cls = rb_define_class("CFormats", rb_cObject);
  rb_define_method(cls, "numbers", numbers, 0);
  rb_define_method(cls, "floats", floats, 0);
  rb_define_method(cls, "texts", texts, 1);
  rb_define_method(cls, "pointer_text", pointer_text, 0);
  rb_define_method(cls, "unknown", unknown, 0);
  rb_define_method(cls, "appended", appended, 1);
  rb_define_method(cls, "raised", raised, 2);
  rb_define_method(cls, "warned", warned, 1);
  rb_define_method(cls, "bytes", bytes, 0);
  rb_define_method(cls, "as_strings", as_strings, 1);
}
