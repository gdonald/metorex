#ifndef RUBY_DIGEST_H
#define RUBY_DIGEST_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

#define RUBY_DIGEST_API_VERSION 3

typedef int (*rb_digest_hash_init_func_t)(void *context);
typedef void (*rb_digest_hash_update_func_t)(void *context, unsigned char *bytes, size_t length);
typedef int (*rb_digest_hash_finish_func_t)(void *context, unsigned char *digest);

typedef struct {
  int api_version;
  size_t digest_len;
  size_t block_len;
  size_t ctx_size;
  rb_digest_hash_init_func_t init_func;
  rb_digest_hash_update_func_t update_func;
  rb_digest_hash_finish_func_t finish_func;
} rb_digest_metadata_t;

/* The Digest module, loaded when it is not yet, with Digest::Base ready to
 * run the plugin a subclass names in its "metadata" instance variable. */
VALUE rb_digest_namespace(void);

static inline VALUE rb_digest_make_metadata(const rb_digest_metadata_t *metadata) {
  return rb_obj_freeze(Data_Wrap_Struct(rb_cObject, 0, 0, (void *)metadata));
}

#ifdef __cplusplus
}
#endif

#endif
