#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

void free_service(void *svc);

void *new_service(uint32_t width, uint32_t height, const uint8_t *endpoint);

void send_frame(void *svc, const uint8_t *data);

} // extern "C"
