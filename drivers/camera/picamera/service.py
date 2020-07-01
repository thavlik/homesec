from ctypes import ctypes

class Service:
    def __init__(self, width: int, height: int, endpoint: str, libpath="/usr/lib/libcamera.so"):
        self.lib = ctypes.CDLL(libpath)
        self.impl = self.lib.new_service(width, height, endpoint)

    def send_frame(self, image):
        self.lib.send_frame(self.impl, image)
