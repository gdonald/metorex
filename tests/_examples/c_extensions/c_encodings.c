#include "ruby.h"
#include "ruby/encoding.h"

static VALUE name_of(rb_encoding *encoding) { return encoding ? rb_str_new_cstr(rb_enc_name(encoding)) : Qnil; }
static rb_encoding *encoding_arg(VALUE encoding) { return NIL_P(encoding) ? NULL : rb_to_encoding(encoding); }

static VALUE find(VALUE self, VALUE name) { return name_of(rb_enc_find(StringValueCStr(name))); }
static VALUE find_index(VALUE self, VALUE name) { return INT2FIX(rb_enc_find_index(StringValueCStr(name))); }
static VALUE from_index(VALUE self, VALUE index) { return name_of(rb_enc_from_index(FIX2INT(index))); }
static VALUE to_index(VALUE self, VALUE encoding) { return INT2FIX(rb_enc_to_index(encoding_arg(encoding))); }
static VALUE to_encoding_index(VALUE self, VALUE encoding) { return INT2FIX(rb_to_encoding_index(encoding)); }
static VALUE indexes(VALUE self) {
  return rb_ary_new_from_args(5, INT2FIX(rb_ascii8bit_encindex()), INT2FIX(rb_utf8_encindex()),
                              INT2FIX(rb_usascii_encindex()), INT2FIX(rb_locale_encindex()),
                              INT2FIX(rb_filesystem_encindex()));
}
static VALUE settings(VALUE self) {
  return rb_ary_new_from_args(4, name_of(rb_locale_encoding()), name_of(rb_filesystem_encoding()),
                              name_of(rb_default_internal_encoding()), name_of(rb_default_external_encoding()));
}
static VALUE alias(VALUE self, VALUE name, VALUE original) {
  return INT2FIX(rb_enc_alias(StringValueCStr(name), StringValueCStr(original)));
}
static VALUE define_dummy(VALUE self, VALUE name) { return INT2FIX(rb_define_dummy_encoding(StringValueCStr(name))); }
static VALUE get(VALUE self, VALUE object) { return name_of(rb_enc_get(object)); }
static VALUE get_index(VALUE self, VALUE object) { return INT2FIX(ENCODING_GET(object)); }
static VALUE set_index(VALUE self, VALUE object, VALUE index) {
  ENCODING_SET(object, FIX2INT(index));
  return object;
}
static VALUE associate(VALUE self, VALUE object, VALUE encoding) { return rb_enc_associate(object, encoding_arg(encoding)); }
static VALUE associate_index(VALUE self, VALUE object, VALUE index) { return rb_enc_associate_index(object, FIX2INT(index)); }
static VALUE copy(VALUE self, VALUE destination, VALUE source) {
  rb_enc_copy(destination, source);
  return destination;
}
static VALUE obj_encoding(VALUE self, VALUE object) { return rb_obj_encoding(object); }
static VALUE compatible(VALUE self, VALUE first, VALUE second) { return name_of(rb_enc_compatible(first, second)); }
static VALUE check(VALUE self, VALUE first, VALUE second) { return name_of(rb_enc_check(first, second)); }
static VALUE str_new(VALUE self, VALUE bytes, VALUE encoding) {
  return rb_enc_str_new(RSTRING_PTR(bytes), RSTRING_LEN(bytes), encoding_arg(encoding));
}
static VALUE str_new_cstr(VALUE self, VALUE encoding) { return rb_enc_str_new_cstr("literal", encoding_arg(encoding)); }
static VALUE str_new_static(VALUE self) { return rb_enc_str_new_static("static", 6, NULL); }
static VALUE coderange(VALUE self, VALUE string) {
  switch (rb_enc_str_coderange(string)) {
  case ENC_CODERANGE_7BIT: return ID2SYM(rb_intern("seven_bit"));
  case ENC_CODERANGE_VALID: return ID2SYM(rb_intern("valid"));
  default: return ID2SYM(rb_intern("broken"));
  }
}
static VALUE ascii_only(VALUE self, VALUE string) {
  RB_ENC_CODERANGE_CLEAR(string);
  return rb_enc_str_asciionly_p(string) ? Qtrue : Qfalse;
}
static VALUE codelen(VALUE self, VALUE code, VALUE encoding) { return INT2FIX(rb_enc_codelen(FIX2INT(code), encoding_arg(encoding))); }
static VALUE mbcput(VALUE self, VALUE code, VALUE encoding) {
  char buffer[ONIGENC_CODE_TO_MBC_MAXLEN];
  int length = rb_enc_mbcput(FIX2UINT(code), buffer, encoding_arg(encoding));
  return rb_enc_str_new(buffer, length, encoding_arg(encoding));
}
static VALUE str_length(VALUE self, VALUE string, VALUE length) {
  return LONG2NUM(rb_enc_strlen(RSTRING_PTR(string), RSTRING_PTR(string) + FIX2INT(length), rb_enc_get(string)));
}
static VALUE to_codepoint(VALUE self, VALUE string) {
  return UINT2NUM(rb_enc_mbc_to_codepoint(RSTRING_PTR(string), RSTRING_END(string), rb_enc_get(string)));
}
static VALUE precise_length(VALUE self, VALUE string) {
  int length = rb_enc_precise_mbclen(RSTRING_PTR(string), RSTRING_END(string), rb_enc_get(string));
  return rb_ary_new_from_args(4, INT2FIX(length), MBCLEN_CHARFOUND_P(length) ? Qtrue : Qfalse,
                              MBCLEN_NEEDMORE_P(length) ? INT2FIX(MBCLEN_NEEDMORE_LEN(length)) : Qnil,
                              MBCLEN_INVALID_P(length) ? Qtrue : Qfalse);
}
static VALUE nth(VALUE self, VALUE string, VALUE index) {
  char *start = RSTRING_PTR(string);
  return LONG2NUM(rb_enc_nth(start, RSTRING_END(string), FIX2LONG(index), rb_enc_get(string)) - start);
}
static VALUE codepoint_length(VALUE self, VALUE string) {
  int length = 0;
  unsigned int code = rb_enc_codepoint_len(RSTRING_PTR(string), RSTRING_END(string), &length, rb_enc_get(string));
  return rb_ary_new_from_args(2, UINT2NUM(code), INT2FIX(length));
}
static VALUE left_head(VALUE self, VALUE string, VALUE offset) {
  char *start = RSTRING_PTR(string);
  return LONG2NUM(rb_enc_left_char_head(start, start + FIX2INT(offset), RSTRING_END(string), rb_enc_get(string)) - start);
}
static VALUE classes(VALUE self, VALUE code, VALUE encoding) {
  rb_encoding *held = encoding_arg(encoding);
  return rb_ary_new_from_args(2, rb_enc_isalnum(FIX2INT(code), held) ? Qtrue : Qfalse,
                              rb_enc_isspace(FIX2INT(code), held) ? Qtrue : Qfalse);
}
static VALUE uv_to_utf8(VALUE self, VALUE code) {
  char buffer[6];
  int length = rb_uv_to_utf8(buffer, NUM2ULONG(code));
  return rb_str_new(buffer, length);
}
static VALUE enc_raise(VALUE self, VALUE encoding, VALUE klass, VALUE message) {
  rb_enc_raise(encoding_arg(encoding), klass, "%s", StringValueCStr(message));
  return Qnil;
}
static VALUE case_fold(VALUE self, VALUE string) {
  OnigUChar folded[ONIGENC_GET_CASE_FOLD_CODES_MAX_NUM];
  const char *at = RSTRING_PTR(string);
  const char *start = at;
  rb_encoding *encoding = rb_enc_get(string);
  int length = ONIGENC_MBC_CASE_FOLD(encoding, ONIGENC_CASE_FOLD, &at, (const OnigUChar *)RSTRING_END(string), folded);
  return rb_ary_new_from_args(2, rb_enc_str_new((char *)folded, length, encoding), LONG2NUM(at - start));
}
static VALUE is_unicode(VALUE self, VALUE encoding) { return ONIGENC_IS_UNICODE(encoding_arg(encoding)) ? Qtrue : Qfalse; }

void Init_c_encodings(void) {
  VALUE cls = rb_define_class("CEncodings", rb_cObject);
  rb_define_method(cls, "find", find, 1);
  rb_define_method(cls, "find_index", find_index, 1);
  rb_define_method(cls, "from_index", from_index, 1);
  rb_define_method(cls, "to_index", to_index, 1);
  rb_define_method(cls, "to_encoding_index", to_encoding_index, 1);
  rb_define_method(cls, "indexes", indexes, 0);
  rb_define_method(cls, "settings", settings, 0);
  rb_define_method(cls, "alias", alias, 2);
  rb_define_method(cls, "define_dummy", define_dummy, 1);
  rb_define_method(cls, "get", get, 1);
  rb_define_method(cls, "get_index", get_index, 1);
  rb_define_method(cls, "set_index", set_index, 2);
  rb_define_method(cls, "associate", associate, 2);
  rb_define_method(cls, "associate_index", associate_index, 2);
  rb_define_method(cls, "copy", copy, 2);
  rb_define_method(cls, "obj_encoding", obj_encoding, 1);
  rb_define_method(cls, "compatible", compatible, 2);
  rb_define_method(cls, "check", check, 2);
  rb_define_method(cls, "str_new", str_new, 2);
  rb_define_method(cls, "str_new_cstr", str_new_cstr, 1);
  rb_define_method(cls, "str_new_static", str_new_static, 0);
  rb_define_method(cls, "coderange", coderange, 1);
  rb_define_method(cls, "ascii_only", ascii_only, 1);
  rb_define_method(cls, "codelen", codelen, 2);
  rb_define_method(cls, "mbcput", mbcput, 2);
  rb_define_method(cls, "str_length", str_length, 2);
  rb_define_method(cls, "to_codepoint", to_codepoint, 1);
  rb_define_method(cls, "precise_length", precise_length, 1);
  rb_define_method(cls, "nth", nth, 2);
  rb_define_method(cls, "codepoint_length", codepoint_length, 1);
  rb_define_method(cls, "left_head", left_head, 2);
  rb_define_method(cls, "classes", classes, 2);
  rb_define_method(cls, "uv_to_utf8", uv_to_utf8, 1);
  rb_define_method(cls, "enc_raise", enc_raise, 3);
  rb_define_method(cls, "case_fold", case_fold, 1);
  rb_define_method(cls, "is_unicode", is_unicode, 1);
}
