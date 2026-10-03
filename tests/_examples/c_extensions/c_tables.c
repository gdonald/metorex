#include "ruby.h"
#include "ruby/st.h"

static int collect(st_data_t key, st_data_t value, st_data_t seen) {
  rb_ary_push((VALUE)seen, LONG2NUM((long)(key * 100 + value)));
  if (key == 3) return ST_STOP;
  if (key == 2) return ST_DELETE;
  return ST_CONTINUE;
}

static VALUE numbers(VALUE self) {
  VALUE answers = rb_ary_new();
  st_table *table = st_init_numtable();
  st_data_t value = 0;
  rb_ary_push(answers, INT2FIX(st_insert(table, 1, 10)));
  rb_ary_push(answers, INT2FIX(st_insert(table, 1, 11)));
  st_add_direct(table, 2, 20);
  st_insert(table, 3, 30);
  st_insert(table, 4, 40);
  rb_ary_push(answers, INT2FIX(st_lookup(table, 1, &value)));
  rb_ary_push(answers, LONG2NUM((long)value));
  rb_ary_push(answers, INT2FIX(st_is_member(table, 9)));
  rb_ary_push(answers, LONG2NUM((long)table->num_entries));
  VALUE seen = rb_ary_new();
  st_foreach(table, collect, (st_data_t)seen);
  rb_ary_push(answers, seen);
  rb_ary_push(answers, LONG2NUM((long)st_table_size(table)));
  st_data_t key = 4;
  rb_ary_push(answers, INT2FIX(st_delete(table, &key, &value)));
  rb_ary_push(answers, LONG2NUM((long)value));
  key = 4;
  value = 99;
  rb_ary_push(answers, INT2FIX(st_delete(table, &key, &value)));
  rb_ary_push(answers, LONG2NUM((long)value));
  key = 1;
  rb_ary_push(answers, INT2FIX(st_delete(table, &key, NULL)));
  key = 1;
  rb_ary_push(answers, INT2FIX(st_delete(table, &key, NULL)));
  st_clear(table);
  rb_ary_push(answers, LONG2NUM((long)table->num_entries));
  st_free_table(table);
  return answers;
}

static VALUE strings(VALUE self) {
  VALUE answers = rb_ary_new();
  st_table *exact = st_init_strtable_with_size(4);
  st_insert(exact, (st_data_t)"Key", 1);
  rb_ary_push(answers, INT2FIX(st_lookup(exact, (st_data_t)"Key", NULL)));
  rb_ary_push(answers, INT2FIX(st_lookup(exact, (st_data_t)"key", NULL)));
  st_free_table(exact);
  st_table *folded = st_init_strcasetable();
  st_insert(folded, (st_data_t)"Key", 1);
  rb_ary_push(answers, INT2FIX(st_lookup(folded, (st_data_t)"kEY", NULL)));
  st_free_table(folded);
  st_free_table(st_init_strtable());
  st_free_table(st_init_strcasetable_with_size(4));
  st_free_table(st_init_numtable_with_size(4));
  return answers;
}

static int same_parity(st_data_t first, st_data_t second) { return (first % 2) != (second % 2); }
static st_index_t parity(st_data_t key) { return key % 2; }
static const struct st_hash_type parity_type = {same_parity, parity};

static VALUE custom(VALUE self) {
  VALUE answers = rb_ary_new();
  st_table *table = st_init_table(&parity_type);
  st_insert(table, 2, 1);
  rb_ary_push(answers, INT2FIX(st_insert(table, 4, 2)));
  rb_ary_push(answers, LONG2NUM((long)table->num_entries));
  st_free_table(table);
  st_free_table(st_init_table_with_size(&parity_type, 8));
  return answers;
}

void Init_c_tables(void) {
  VALUE cls = rb_define_class("CTables", rb_cObject);
  rb_define_method(cls, "numbers", numbers, 0);
  rb_define_method(cls, "strings", strings, 0);
  rb_define_method(cls, "custom", custom, 0);
}
