#include "ruby.h"

static VALUE hash_code(VALUE self, VALUE object) { return rb_hash(object); }
static VALUE convert(VALUE self, VALUE object) { return rb_Hash(object); }
static VALUE made(VALUE self) {
  return rb_ary_new_from_args(3, rb_hash_new(), rb_hash_new_capa(8), rb_ident_hash_new());
}
static VALUE negative_capacity(VALUE self) { return rb_hash_new_capa(-1); }
static VALUE frozen(VALUE self, VALUE hash) { return rb_hash_freeze(hash); }
static VALUE read_entry(VALUE self, VALUE hash, VALUE key) {
  return rb_ary_new_from_args(3, rb_hash_aref(hash, key), rb_hash_lookup(hash, key),
                              rb_hash_lookup2(hash, key, ID2SYM(rb_intern("missing"))));
}
static VALUE missing_is_undef(VALUE self, VALUE hash, VALUE key) {
  return rb_hash_lookup2(hash, key, Qundef) == Qundef ? Qtrue : Qfalse;
}
static VALUE write_entry(VALUE self, VALUE hash, VALUE key, VALUE value) { return rb_hash_aset(hash, key, value); }
static VALUE remove_entry(VALUE self, VALUE hash, VALUE key) { return rb_hash_delete(hash, key); }
static VALUE remove_if(VALUE self, VALUE hash) { return rb_hash_delete_if(hash); }
static VALUE fetch(VALUE self, VALUE hash, VALUE key) { return rb_hash_fetch(hash, key); }
static VALUE emptied(VALUE self, VALUE hash) { return rb_hash_clear(hash); }
static VALUE size(VALUE self, VALUE hash) { return rb_hash_size(hash); }
static VALUE set_default(VALUE self, VALUE hash, VALUE value) { return rb_hash_set_ifnone(hash, value); }
static VALUE insert(VALUE self, VALUE pairs, VALUE hash) {
  rb_hash_bulk_insert(RARRAY_LEN(pairs), RARRAY_PTR(pairs), hash);
  return hash;
}

static int copy_each(VALUE key, VALUE value, VALUE copy) {
  rb_hash_aset(copy, key, value);
  return ST_CONTINUE;
}
static int copy_first(VALUE key, VALUE value, VALUE copy) {
  rb_hash_aset(copy, key, value);
  return ST_STOP;
}
static int copy_and_remove(VALUE key, VALUE value, VALUE copy) {
  rb_hash_aset(copy, key, value);
  return ST_DELETE;
}
static VALUE walk(VALUE self, VALUE hash, VALUE how) {
  VALUE copy = rb_hash_new();
  const char *named = rb_id2name(SYM2ID(how));
  if (strcmp(named, "each") == 0) rb_hash_foreach(hash, copy_each, copy);
  if (strcmp(named, "first") == 0) rb_hash_foreach(hash, copy_first, copy);
  if (strcmp(named, "remove") == 0) rb_hash_foreach(hash, copy_and_remove, copy);
  return copy;
}

static VALUE mixed(VALUE self, VALUE seed) {
  st_index_t code = rb_hash_start(FIX2INT(seed));
  code = rb_hash_uint32(code, 540u);
  code = rb_hash_uint(code, 340u);
  return ULONG2NUM(rb_hash_end(code));
}

void Init_c_hashes(void) {
  VALUE klass = rb_define_class("CHashes", rb_cObject);
  rb_define_method(klass, "hash_code", hash_code, 1);
  rb_define_method(klass, "convert", convert, 1);
  rb_define_method(klass, "made", made, 0);
  rb_define_method(klass, "negative_capacity", negative_capacity, 0);
  rb_define_method(klass, "frozen", frozen, 1);
  rb_define_method(klass, "read", read_entry, 2);
  rb_define_method(klass, "missing_is_undef", missing_is_undef, 2);
  rb_define_method(klass, "write", write_entry, 3);
  rb_define_method(klass, "remove", remove_entry, 2);
  rb_define_method(klass, "remove_if", remove_if, 1);
  rb_define_method(klass, "fetch", fetch, 2);
  rb_define_method(klass, "emptied", emptied, 1);
  rb_define_method(klass, "size", size, 1);
  rb_define_method(klass, "set_default", set_default, 2);
  rb_define_method(klass, "insert", insert, 2);
  rb_define_method(klass, "walk", walk, 2);
  rb_define_method(klass, "mixed", mixed, 1);
}
