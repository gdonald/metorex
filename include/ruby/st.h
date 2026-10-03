#ifndef RUBY_ST_H
#define RUBY_ST_H 1

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uintptr_t st_data_t;
typedef st_data_t st_index_t;

typedef int st_compare_func(st_data_t, st_data_t);
typedef st_index_t st_hash_func(st_data_t);

struct st_hash_type {
  st_compare_func *compare;
  st_hash_func *hash;
};

typedef struct st_table {
  const struct st_hash_type *type;
  st_index_t num_entries;
  void *metorex_entries;
} st_table;

enum st_retval { ST_CONTINUE, ST_STOP, ST_DELETE, ST_CHECK, ST_REPLACE };

typedef int st_foreach_callback_func(st_data_t key, st_data_t value, st_data_t data);

st_table *rb_st_init_table(const struct st_hash_type *type);
st_table *rb_st_init_table_with_size(const struct st_hash_type *type, st_index_t size);
st_table *rb_st_init_numtable(void);
st_table *rb_st_init_numtable_with_size(st_index_t size);
st_table *rb_st_init_strtable(void);
st_table *rb_st_init_strtable_with_size(st_index_t size);
st_table *rb_st_init_strcasetable(void);
st_table *rb_st_init_strcasetable_with_size(st_index_t size);
void rb_st_free_table(st_table *table);
void rb_st_clear(st_table *table);
size_t rb_st_table_size(const st_table *table);
int rb_st_insert(st_table *table, st_data_t key, st_data_t value);
void rb_st_add_direct(st_table *table, st_data_t key, st_data_t value);
int rb_st_lookup(st_table *table, st_data_t key, st_data_t *value);
int rb_st_delete(st_table *table, st_data_t *key, st_data_t *value);
int rb_st_foreach(st_table *table, st_foreach_callback_func *function, st_data_t data);

#define st_init_table rb_st_init_table
#define st_init_table_with_size rb_st_init_table_with_size
#define st_init_numtable rb_st_init_numtable
#define st_init_numtable_with_size rb_st_init_numtable_with_size
#define st_init_strtable rb_st_init_strtable
#define st_init_strtable_with_size rb_st_init_strtable_with_size
#define st_init_strcasetable rb_st_init_strcasetable
#define st_init_strcasetable_with_size rb_st_init_strcasetable_with_size
#define st_free_table rb_st_free_table
#define st_clear rb_st_clear
#define st_table_size rb_st_table_size
#define st_insert rb_st_insert
#define st_add_direct rb_st_add_direct
#define st_lookup rb_st_lookup
#define st_delete rb_st_delete
#define st_foreach rb_st_foreach
#define st_is_member(table, key) st_lookup((table), (key), (st_data_t *)0)

#ifdef __cplusplus
}
#endif

#endif
