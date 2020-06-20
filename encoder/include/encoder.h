#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

int32_t encode_frame(void *encoder, const char *frame);

void free_encoder(void *encoder);

void *new_encoder(int32_t width, int32_t height);

} // extern "C"
