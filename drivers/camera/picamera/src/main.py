import os
import argparse
import time
from picamera.array import PiRGBArray
from picamera import PiCamera
from service import Service

parser = argparse.ArgumentParser(description='VAE MNIST Example')
parser.add_argument('--width', type=int, default=1920, metavar='w',
                    help='horizontal resolution (in pixels)')
parser.add_argument('--height', type=int, default=1080, metavar='h',
                    help='vertical resolution (in pixels)')
parser.add_argument('--frame-rate', type=int, default=30, metavar='r',
                    help='frames per second')
parser.add_argument('--lib-path', type=str,
                    default=os.path.normpath(os.path.join(
                        os.path.realpath(__file__), "libcamera.so")),
                    help='path to libcamera.so')

args = parser.parse_args()

resolution = (args.width, args.height)
camera = PiCamera()
camera.resolution = resolution
camera.framerate = args.frame_rate
raw_capture = PiRGBArray(camera, size=resolution)
time.sleep(0.1)
with Service(width=args.width, height=args.height, endpoint='') as svc:
    for frame in camera.capture_continuous(raw_capture, format="bgr", use_video_port=True):
        image = frame.array
        svc.send_frame(image)
        print(image[:128])
        raw_capture.truncate(0)

