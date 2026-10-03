#include "ruby.h"
#include "ruby/re.h"

static VALUE make(VALUE self, VALUE source, VALUE options) {
  return rb_reg_new(RSTRING_PTR(source), RSTRING_LEN(source), FIX2INT(options));
}
static VALUE compile(VALUE self, VALUE source) { return rb_reg_regcomp(source); }
static VALUE options(VALUE self, VALUE regexp) { return INT2FIX(rb_reg_options(regexp)); }
static VALUE match(VALUE self, VALUE regexp, VALUE string) { return rb_reg_match(regexp, string); }
static VALUE nth(VALUE self, VALUE index, VALUE matched) { return rb_reg_nth_match(FIX2INT(index), matched); }
static VALUE last(VALUE self) { return rb_backref_get(); }
static VALUE set_last(VALUE self, VALUE matched) {
  rb_backref_set(matched);
  return Qnil;
}
static VALUE compare(VALUE self, VALUE first, VALUE second, VALUE length) {
  return INT2FIX(rb_memcicmp(RSTRING_PTR(first), RSTRING_PTR(second), FIX2LONG(length)));
}
static VALUE string_value(VALUE self, VALUE value) {
  StringValue(value);
  return value;
}
static VALUE string_pointer(VALUE self, VALUE value) {
  char *pointer = StringValuePtr(value);
  return INT2FIX((unsigned char)pointer[0]);
}
static VALUE c_string_length(VALUE self, VALUE value) {
  return LONG2NUM((long)strlen(StringValueCStr(value)));
}

void Init_c_regexps(void) {
  VALUE cls = rb_define_class("CRegexps", rb_cObject);
  rb_define_method(cls, "make", make, 2);
  rb_define_method(cls, "compile", compile, 1);
  rb_define_method(cls, "options", options, 1);
  rb_define_method(cls, "match", match, 2);
  rb_define_method(cls, "nth", nth, 2);
  rb_define_method(cls, "last", last, 0);
  rb_define_method(cls, "set_last", set_last, 1);
  rb_define_method(cls, "compare", compare, 3);
  rb_define_method(cls, "string_value", string_value, 1);
  rb_define_method(cls, "string_pointer", string_pointer, 1);
  rb_define_method(cls, "c_string_length", c_string_length, 1);
}
