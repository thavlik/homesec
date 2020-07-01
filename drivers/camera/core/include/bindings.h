#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct Service;

extern "C" {

void free_service(Service *svc);

Service *new_service(uint32_t width, uint32_t height, const char *endpoint);

void send_frame(Service *svc, const uint8_t *data);

} // extern "C"
