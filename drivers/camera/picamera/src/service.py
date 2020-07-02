from ctypes import *

class Service:
    def __init__(self,
                 width: int,
                 height: int,
                 endpoint: str,
                 dylibpath="/usr/lib/libcamera_core.so"):
        self.lib = CDLL(dylibpath)
        self.impl = self.lib.new_service(width, height, endpoint)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.lib.free_service(self.impl)

    def send_frame(self, image):
        self.lib.send_frame(self.impl, image.ctypes.data)
