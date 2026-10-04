#include "ruby.h"
#include <stdio.h>

struct node {
  int id;
  VALUE child;
};

static VALUE freed;
static VALUE pinned;

static void node_mark(void *pointer) {
  struct node *held = pointer;
  rb_gc_mark(held->child);
  rb_gc_mark_movable(held->child);
  rb_gc_mark_maybe(held->child);
  VALUE both[2] = {held->child, Qnil};
  rb_gc_mark_locations(both, both + 2);
  held->child = rb_gc_location(held->child);
}

static void node_free(void *pointer) {
  struct node *held = pointer;
  if (NIL_P(freed)) {
    printf("freed %d as the program ended\n", held->id);
    fflush(stdout);
  } else {
    rb_ary_push(freed, INT2FIX(held->id));
  }
  free(held);
}

static const rb_data_type_t node_type = {
  "node",
  {node_mark, node_free, NULL},
  NULL,
  NULL,
  0
};

static VALUE make_node(VALUE self, VALUE id, VALUE child) {
  struct node *held = malloc(sizeof(struct node));
  held->id = FIX2INT(id);
  held->child = child;
  return TypedData_Wrap_Struct(rb_cObject, &node_type, held);
}

static VALUE make_plain(VALUE self) {
  int *held = malloc(sizeof(int));
  *held = 7;
  return Data_Wrap_Struct(rb_cObject, NULL, RUBY_DEFAULT_FREE, held);
}

static VALUE pin(VALUE self, VALUE node) {
  pinned = node;
  return Qnil;
}

static VALUE freed_ids(VALUE self) { return rb_ary_dup(freed); }

static VALUE stop_recording(VALUE self) {
  freed = Qnil;
  return Qnil;
}

void Init_c_marking(void) {
  VALUE module = rb_define_module("CMarking");
  freed = rb_ary_new();
  rb_gc_register_address(&freed);
  pinned = Qnil;
  rb_gc_register_address(&pinned);
  rb_define_singleton_method(module, "node", make_node, 2);
  rb_define_singleton_method(module, "plain", make_plain, 0);
  rb_define_singleton_method(module, "pin", pin, 1);
  rb_define_singleton_method(module, "freed", freed_ids, 0);
  rb_define_singleton_method(module, "stop_recording", stop_recording, 0);
}
