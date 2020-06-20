from picamera.array import PiRGBArray
from picamera import PiCamera
import time
from ctypes import ctypes

so_file = "target/debug/libencoder.so"
rav1e = ctypes.CDLL(so_file)

encoder = rav1e.new_encoder(1920, 1080)

resolution = (1920, 1080)
camera = PiCamera()
camera.resolution = resolution
camera.framerate = 32
rawCapture = PiRGBArray(camera, size=resolution)
time.sleep(0.1)
for frame in camera.capture_continuous(rawCapture, format="bgr", use_video_port=True):
	image = frame.array
	rav1e.encode_frame(encoder, image)
	rawCapture.truncate(0)
