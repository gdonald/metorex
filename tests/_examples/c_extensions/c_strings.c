#include "ruby.h"
#include "ruby/encoding.h"

static rb_encoding *encoding_arg(VALUE encoding) { return NIL_P(encoding) ? NULL : rb_to_encoding(encoding); }

static VALUE set_len(VALUE self, VALUE string, VALUE length) {
  rb_str_set_len(string, NUM2LONG(length));
  return string;
}
static VALUE poke(VALUE self, VALUE string, VALUE index, VALUE byte) {
  RSTRING_PTR(string)[FIX2INT(index)] = (char)FIX2INT(byte);
  return string;
}
static VALUE peek(VALUE self, VALUE string, VALUE index, VALUE count) {
  return rb_str_new(RSTRING_PTR(string) + FIX2INT(index), FIX2INT(count));
}
static VALUE capacity(VALUE self, VALUE string) { return SIZET2NUM(rb_str_capacity(string)); }
static VALUE modify(VALUE self, VALUE string) {
  rb_str_modify(string);
  return string;
}
static VALUE modify_expand(VALUE self, VALUE string, VALUE expand) {
  rb_str_modify_expand(string, NUM2LONG(expand));
  return SIZET2NUM(rb_str_capacity(string));
}
static VALUE resize(VALUE self, VALUE string, VALUE length) { return rb_str_resize(string, NUM2LONG(length)); }
static VALUE buf_new(VALUE self, VALUE room) { return rb_str_buf_new(NUM2LONG(room)); }
static VALUE write_then_yield(VALUE self, VALUE string) {
  char *bytes = RSTRING_PTR(string);
  bytes[0] = 'X';
  rb_yield(string);
  bytes = RSTRING_PTR(string);
  bytes[2] = 'Z';
  return rb_str_new(bytes, RSTRING_LEN(string));
}
static VALUE static_pointer(VALUE self) {
  static const char text[] = "constant";
  VALUE made = rb_utf8_str_new_static(text, 8);
  return rb_ary_new_from_args(2, made, (RSTRING_PTR(made) == text) ? Qtrue : Qfalse);
}
static VALUE usascii_literal(VALUE self) { return rb_usascii_str_new_lit("literal"); }
static VALUE constructors(VALUE self) {
  return rb_ary_new_from_args(6, rb_usascii_str_new("ascii", 5), rb_usascii_str_new_cstr("ascii"), rb_utf8_str_new("utf", 3),
                              rb_utf8_str_new_cstr("utf"), rb_str_buf_new2("buffer"), rb_external_str_new_cstr("outside"));
}
static VALUE external_with(VALUE self, VALUE bytes, VALUE encoding) {
  return rb_external_str_new_with_enc(RSTRING_PTR(bytes), RSTRING_LEN(bytes), encoding_arg(encoding));
}
static VALUE locale_strings(VALUE self) { return rb_ary_new_from_args(2, rb_locale_str_new("here", 4), rb_locale_str_new_cstr("here")); }
static VALUE tmp_class(VALUE self) {
  VALUE made = rb_str_tmp_new(3);
  VALUE hidden = RBASIC_CLASS(made);
  rb_obj_reveal(made, rb_cString);
  return rb_ary_new_from_args(3, hidden, RBASIC_CLASS(made), made);
}
static VALUE copies(VALUE self, VALUE string) {
  return rb_ary_new_from_args(4, rb_str_dup(string), rb_str_new3(string), rb_str_new4(string), rb_str_new5(string, "made", 4));
}
static VALUE drop_bytes(VALUE self, VALUE string, VALUE count) { return rb_str_drop_bytes(string, NUM2LONG(count)); }
static VALUE free_string(VALUE self, VALUE string) {
  rb_str_free(string);
  return Qnil;
}
static VALUE lock(VALUE self, VALUE string) { return rb_str_locktmp(string); }
static VALUE unlock(VALUE self, VALUE string) { return rb_str_unlocktmp(string); }
static VALUE interned(VALUE self, VALUE bytes, VALUE encoding) {
  if (encoding == Qfalse) return rb_interned_str(RSTRING_PTR(bytes), RSTRING_LEN(bytes));
  return rb_enc_interned_str(RSTRING_PTR(bytes), RSTRING_LEN(bytes), encoding_arg(encoding));
}
static VALUE interned_cstr(VALUE self, VALUE bytes, VALUE encoding) {
  if (encoding == Qfalse) return rb_interned_str_cstr(StringValueCStr(bytes));
  return rb_enc_interned_str_cstr(StringValueCStr(bytes), encoding_arg(encoding));
}
static VALUE to_interned(VALUE self, VALUE string) { return rb_str_to_interned_str(string); }
static VALUE joins(VALUE self, VALUE first, VALUE second) {
  return rb_ary_new_from_args(3, rb_str_plus(first, second), rb_str_times(first, INT2FIX(2)), rb_str_buf_append(rb_str_dup(first), second));
}
static VALUE append(VALUE self, VALUE string, VALUE added) { return rb_str_append(string, added); }
static VALUE cats(VALUE self, VALUE string) {
  rb_str_buf_cat(string, "-buf", 4);
  rb_str_cat(string, "-cat", 4);
  rb_str_cat2(string, "-cat2");
  return rb_str_cat_cstr(string, "-cstr");
}
static VALUE enc_cat(VALUE self, VALUE string, VALUE added, VALUE encoding) {
  return rb_enc_str_buf_cat(string, RSTRING_PTR(added), RSTRING_LEN(added), encoding_arg(encoding));
}
static VALUE compare(VALUE self, VALUE first, VALUE second) {
  return rb_ary_new_from_args(2, INT2FIX(rb_str_cmp(first, second)), rb_str_equal(first, second));
}
static VALUE lengths(VALUE self, VALUE string, VALUE byte_offset) {
  return rb_ary_new_from_args(3, rb_str_length(string), LONG2NUM(rb_str_strlen(string)),
                              LONG2NUM(rb_str_sublen(string, NUM2LONG(byte_offset))));
}
static VALUE subpos(VALUE self, VALUE string, VALUE start, VALUE length) {
  long held = NUM2LONG(length);
  char *at = rb_str_subpos(string, NUM2LONG(start), &held);
  if (at == NULL) return Qnil;
  return rb_ary_new_from_args(2, LONG2NUM(at - RSTRING_PTR(string)), LONG2NUM(held));
}
static VALUE slices(VALUE self, VALUE string) {
  return rb_ary_new_from_args(2, rb_str_subseq(string, 1, 2), rb_str_substr(string, 1, 2));
}
static VALUE update(VALUE self, VALUE string, VALUE start, VALUE length, VALUE replacement) {
  rb_str_update(string, NUM2LONG(start), NUM2LONG(length), replacement);
  return string;
}
static VALUE split(VALUE self, VALUE string) { return rb_str_split(string, ","); }
static VALUE readings(VALUE self, VALUE string) {
  return rb_ary_new_from_args(4, rb_str_inspect(string), rb_str_intern(string), ST2FIX(rb_str_hash(string)) == ST2FIX(rb_str_hash(rb_str_dup(string))) ? Qtrue : Qfalse,
                              rb_str_freeze(string));
}
static VALUE to_inum(VALUE self, VALUE text, VALUE base, VALUE strict) {
  return rb_cstr_to_inum(StringValueCStr(text), FIX2INT(base), RTEST(strict));
}
static VALUE cstr2inum(VALUE self, VALUE text, VALUE base) { return rb_cstr2inum(StringValueCStr(text), FIX2INT(base)); }
static VALUE str2inum(VALUE self, VALUE text, VALUE base) { return rb_str2inum(text, FIX2INT(base)); }
static VALUE encode(VALUE self, VALUE string, VALUE encoding, VALUE flags, VALUE options) {
  return rb_str_encode(string, encoding, FIX2INT(flags), options);
}
static VALUE conv(VALUE self, VALUE string, VALUE from, VALUE to) {
  return rb_str_conv_enc(string, encoding_arg(from), encoding_arg(to));
}
static VALUE conv_opts(VALUE self, VALUE string, VALUE from, VALUE to) {
  return rb_str_conv_enc_opts(string, encoding_arg(from), encoding_arg(to), 0, Qnil);
}
static VALUE exports(VALUE self, VALUE string, VALUE encoding) {
  return rb_ary_new_from_args(3, rb_str_export(string), rb_str_export_locale(string), rb_str_export_to_enc(string, encoding_arg(encoding)));
}
static VALUE string_of(VALUE self, VALUE object) { return rb_String(object); }
static VALUE to_str(VALUE self, VALUE object) { return rb_str_to_str(object); }
static VALUE formatted(VALUE self, VALUE value) { return rb_sprintf("[%-6" PRIsVALUE "|%.2" PRIsVALUE "|%+" PRIsVALUE "]", value, value, value); }
static VALUE vcatf_helper(VALUE string, const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE answered = rb_str_vcatf(string, format, arguments);
  va_end(arguments);
  return answered;
}
static VALUE vcatf(VALUE self, VALUE string) { return vcatf_helper(string, "%d-%s", 7, "seven"); }

void Init_c_strings(void) {
  VALUE cls = rb_define_class("CStrings", rb_cObject);
  rb_define_method(cls, "set_len", set_len, 2);
  rb_define_method(cls, "poke", poke, 3);
  rb_define_method(cls, "peek", peek, 3);
  rb_define_method(cls, "capacity", capacity, 1);
  rb_define_method(cls, "modify", modify, 1);
  rb_define_method(cls, "modify_expand", modify_expand, 2);
  rb_define_method(cls, "resize", resize, 2);
  rb_define_method(cls, "buf_new", buf_new, 1);
  rb_define_method(cls, "write_then_yield", write_then_yield, 1);
  rb_define_method(cls, "static_pointer", static_pointer, 0);
  rb_define_method(cls, "usascii_literal", usascii_literal, 0);
  rb_define_method(cls, "constructors", constructors, 0);
  rb_define_method(cls, "external_with", external_with, 2);
  rb_define_method(cls, "locale_strings", locale_strings, 0);
  rb_define_method(cls, "tmp_class", tmp_class, 0);
  rb_define_method(cls, "copies", copies, 1);
  rb_define_method(cls, "drop_bytes", drop_bytes, 2);
  rb_define_method(cls, "free_string", free_string, 1);
  rb_define_method(cls, "lock", lock, 1);
  rb_define_method(cls, "unlock", unlock, 1);
  rb_define_method(cls, "interned", interned, 2);
  rb_define_method(cls, "interned_cstr", interned_cstr, 2);
  rb_define_method(cls, "to_interned", to_interned, 1);
  rb_define_method(cls, "joins", joins, 2);
  rb_define_method(cls, "append", append, 2);
  rb_define_method(cls, "cats", cats, 1);
  rb_define_method(cls, "enc_cat", enc_cat, 3);
  rb_define_method(cls, "compare", compare, 2);
  rb_define_method(cls, "lengths", lengths, 2);
  rb_define_method(cls, "subpos", subpos, 3);
  rb_define_method(cls, "slices", slices, 1);
  rb_define_method(cls, "update", update, 4);
  rb_define_method(cls, "split", split, 1);
  rb_define_method(cls, "readings", readings, 1);
  rb_define_method(cls, "to_inum", to_inum, 3);
  rb_define_method(cls, "cstr2inum", cstr2inum, 2);
  rb_define_method(cls, "str2inum", str2inum, 2);
  rb_define_method(cls, "encode", encode, 4);
  rb_define_method(cls, "conv", conv, 3);
  rb_define_method(cls, "conv_opts", conv_opts, 3);
  rb_define_method(cls, "exports", exports, 2);
  rb_define_method(cls, "string_of", string_of, 1);
  rb_define_method(cls, "to_str", to_str, 1);
  rb_define_method(cls, "formatted", formatted, 1);
  rb_define_method(cls, "vcatf", vcatf, 1);
}
