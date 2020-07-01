from ctypes import ctypes


class Service:
    def __init__(self,
                 width: int,
                 height: int,
                 endpoint: str,
                 dylibpath="/usr/lib/libcamera_core.so"):
        self.lib = ctypes.CDLL(dylibpath)
        self.impl = self.lib.new_service(width, height, endpoint)

    def send_frame(self, image):
        self.lib.send_frame(self.impl, image)
