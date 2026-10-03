#include "ruby.h"
#include "ruby/encoding.h"

static VALUE is_symbol(VALUE self, VALUE value) { return SYMBOL_P(value) ? Qtrue : Qfalse; }
static VALUE from_bytes(VALUE self, VALUE text, VALUE length) {
  return ID2SYM(rb_intern2(RSTRING_PTR(text), FIX2LONG(length)));
}
static VALUE in_encoding(VALUE self, VALUE text, VALUE encoding) {
  return ID2SYM(rb_intern3(RSTRING_PTR(text), RSTRING_LEN(text), rb_enc_get(encoding)));
}
static VALUE constant(VALUE self) { return ID2SYM(rb_intern_const("made_constant")); }
static VALUE zero_ids(VALUE self) {
  VALUE values[2] = {rb_id2name((ID)0) == NULL ? Qtrue : Qfalse, rb_id2str((ID)0)};
  return rb_ary_new_from_values(2, values);
}
static VALUE name_of(VALUE self, VALUE symbol) { return rb_id2str(SYM2ID(symbol)); }
static VALUE from_string(VALUE self, VALUE string) { return ID2SYM(rb_intern_str(string)); }
static VALUE existing(VALUE self, VALUE string) {
  return rb_check_symbol_cstr(RSTRING_PTR(string), RSTRING_LEN(string), rb_enc_get(string));
}
static VALUE kinds(VALUE self, VALUE symbol) {
  ID id = SYM2ID(symbol);
  VALUE values[3] = {rb_is_const_id(id) ? Qtrue : Qfalse, rb_is_instance_id(id) ? Qtrue : Qfalse,
                     rb_is_class_id(id) ? Qtrue : Qfalse};
  return rb_ary_new_from_values(3, values);
}
static VALUE text_of(VALUE self, VALUE symbol) { return rb_sym2str(symbol); }
static VALUE to_symbol(VALUE self, VALUE value) { return rb_to_symbol(value); }
static VALUE encoding_name(VALUE self, VALUE value) {
  rb_encoding *encoding = rb_enc_get(value);
  return encoding ? rb_str_new_cstr(rb_enc_name(encoding)) : Qnil;
}
static VALUE named_encodings(VALUE self) {
  VALUE values[3] = {rb_enc_from_encoding(rb_utf8_encoding()), rb_enc_from_encoding(rb_usascii_encoding()),
                     rb_enc_from_encoding(rb_ascii8bit_encoding())};
  return rb_ary_new_from_values(3, values);
}

void Init_c_symbols(void) {
  VALUE cls = rb_define_class("CSymbols", rb_cObject);
  rb_define_method(cls, "is_symbol", is_symbol, 1);
  rb_define_method(cls, "from_bytes", from_bytes, 2);
  rb_define_method(cls, "in_encoding", in_encoding, 2);
  rb_define_method(cls, "constant", constant, 0);
  rb_define_method(cls, "zero_ids", zero_ids, 0);
  rb_define_method(cls, "name_of", name_of, 1);
  rb_define_method(cls, "from_string", from_string, 1);
  rb_define_method(cls, "existing", existing, 1);
  rb_define_method(cls, "kinds", kinds, 1);
  rb_define_method(cls, "text_of", text_of, 1);
  rb_define_method(cls, "to_symbol", to_symbol, 1);
  rb_define_method(cls, "encoding_name", encoding_name, 1);
  rb_define_method(cls, "named_encodings", named_encodings, 0);
}
