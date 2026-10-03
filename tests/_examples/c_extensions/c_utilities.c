#include "ruby.h"
#include "ruby/util.h"

static VALUE scanned(int count, VALUE *arguments, VALUE self) {
  VALUE first = Qnil, options = Qnil, block = Qnil;
  int used = rb_scan_args(count, arguments, "01:&", &first, &options, &block);
  return rb_ary_new_from_args(5, INT2FIX(used), first, options, block, rb_keyword_given_p() ? Qtrue : Qfalse);
}
static VALUE scanned_last_hash(int count, VALUE *arguments, VALUE self) {
  VALUE first = Qnil, options = Qnil;
  rb_scan_args_kw(RB_SCAN_ARGS_LAST_HASH_KEYWORDS, count, arguments, "01:", &first, &options);
  return rb_ary_new_from_args(2, first, options);
}
static VALUE block_proc(VALUE self) { return rb_block_proc(); }
static VALUE keywords(VALUE self, VALUE hash, VALUE names, VALUE required, VALUE optional) {
  ID ids[8];
  VALUE values[8];
  int total = (int)RARRAY_LEN(names);
  for (int index = 0; index < total; index++) ids[index] = SYM2ID(rb_ary_entry(names, index));
  int found = rb_get_kwargs(hash, ids, FIX2INT(required), FIX2INT(optional), values);
  int stored = FIX2INT(required) + (FIX2INT(optional) < 0 ? -1 - FIX2INT(optional) : FIX2INT(optional));
  VALUE answered = rb_ary_new_from_values(stored, values);
  rb_ary_unshift(answered, INT2FIX(found));
  return answered;
}
static VALUE keywords_present(VALUE self, VALUE hash, VALUE name) {
  ID id = SYM2ID(name);
  return INT2FIX(rb_get_kwargs(hash, &id, 0, 1, NULL));
}
static VALUE stop(VALUE self, VALUE value) { rb_iter_break_value(value); }
static VALUE stop_plain(VALUE self) { rb_iter_break(); }
static VALUE where(VALUE self) {
  return rb_ary_new_from_args(2, rb_str_new_cstr(rb_sourcefile()), INT2FIX(rb_sourceline()));
}
static VALUE narrowed(VALUE self, VALUE number) { return INT2FIX(rb_long2int(NUM2LONG(number))); }
static VALUE parsed(VALUE self, VALUE text) {
  char *end = NULL;
  double value = strtod(RSTRING_PTR(text), &end);
  return rb_ary_new_from_args(2, DBL2NUM(value), rb_str_new_cstr(end));
}
static VALUE copied(VALUE self, VALUE hash) { return rb_hash_dup(hash); }

void Init_c_utilities(void) {
  VALUE cls = rb_define_class("CUtilities", rb_cObject);
  rb_define_method(cls, "scanned", scanned, -1);
  rb_define_method(cls, "scanned_last_hash", scanned_last_hash, -1);
  rb_define_method(cls, "block_proc", block_proc, 0);
  rb_define_method(cls, "keywords", keywords, 4);
  rb_define_method(cls, "keywords_present", keywords_present, 2);
  rb_define_method(cls, "stop", stop, 1);
  rb_define_method(cls, "stop_plain", stop_plain, 0);
  rb_define_method(cls, "where", where, 0);
  rb_define_method(cls, "narrowed", narrowed, 1);
  rb_define_method(cls, "parsed", parsed, 1);
  rb_define_method(cls, "copied", copied, 1);
}
