#include "ruby.h"
#include "ruby/digest.h"

/* A digest whose answer is the bytes it was given, counted and folded into
 * four bytes, which is enough to see each plugin function run. */
typedef struct {
  unsigned int total;
  unsigned char folded[4];
} Fold;

static int fold_init(void *raw) {
  Fold *fold = (Fold *)raw;
  fold->total = 0;
  memset(fold->folded, 0, 4);
  return 1;
}

static void fold_update(void *raw, unsigned char *bytes, size_t length) {
  Fold *fold = (Fold *)raw;
  for (size_t index = 0; index < length; index++) {
    fold->folded[fold->total % 4] ^= bytes[index];
    fold->total++;
  }
}

static int fold_finish(void *raw, unsigned char *digest) {
  Fold *fold = (Fold *)raw;
  memcpy(digest, fold->folded, 4);
  digest[4] = (unsigned char)fold->total;
  return 1;
}

static const rb_digest_metadata_t metadata = {
  RUBY_DIGEST_API_VERSION, 5, 8, sizeof(Fold), fold_init, fold_update, fold_finish,
};

static const char marker[] = "marker";
static VALUE marker_address(VALUE self) { return LONG2NUM((long)marker); }

void Init_c_digests(void) {
  VALUE digest = rb_digest_namespace();
  VALUE base = rb_const_get(digest, rb_intern("Base"));
  VALUE fold = rb_define_class_under(digest, "Fold", base);
  rb_iv_set(fold, "metadata", rb_digest_make_metadata(&metadata));
  VALUE cls = rb_define_class("CDigests", rb_cObject);
  rb_define_method(cls, "marker_address", marker_address, 0);
}
