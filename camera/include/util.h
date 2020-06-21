#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

int32_t encode_frame(void *encoder, const void *frame);

void free_encoder(void *encoder);

int32_t mul(int32_t x, int32_t y);

void *new_encoder(int32_t width, int32_t height);

} // extern "C"
