import os
import argparse
import time
from picamera.array import PiRGBArray
from picamera import PiCamera
from service import Service

parser = argparse.ArgumentParser(description='picamera driver')
parser.add_argument('--width', type=int, default=1920,
                    help='horizontal resolution (in pixels)')
parser.add_argument('--height', type=int, default=1080,
                    help='vertical resolution (in pixels)')
parser.add_argument('--frame-rate', type=int, default=30,
                    help='frames per second')
parser.add_argument('--lib-path', type=str,
                    default='/usr/lib/libcamera_core.so',
                    help='path to libcamera_core.so')
parser.add_argument('--endpoint', type=str,
                    default='',
                    help='transmission endpoint?')

args = parser.parse_args()

resolution = (args.width, args.height)
camera = PiCamera()
camera.resolution = resolution
camera.framerate = args.frame_rate
raw_capture = PiRGBArray(camera, size=resolution)
time.sleep(0.1)
with Service(width=args.width,
             height=args.height,
             endpoint=args.endpoint,
             dylibpath=args.lib_path) as svc:
    for frame in camera.capture_continuous(raw_capture, format="bgr", use_video_port=True):
        image = frame.array
        svc.send_frame(image)
        print(image[:128])
        raw_capture.truncate(0)

