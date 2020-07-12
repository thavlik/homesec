import os
import argparse
import time
from picamera.array import PiRGBArray
from picamera import PiCamera
from service import Service

parser = argparse.ArgumentParser(description='picamera driver')
parser.add_argument('--width', type=int, default=640,
                    help='horizontal resolution (in pixels)')
parser.add_argument('--height', type=int, default=480,
                    help='vertical resolution (in pixels)')
parser.add_argument('--frame-rate', type=int, default=30,
                    help='frames per second')
parser.add_argument('--lib-path', type=str,
                    default='/usr/lib/libcamera_core.so',
                    help='path to libcamera_core.so')
parser.add_argument('--endpoint', type=str,
                    default='192.168.1.100',
                    help='transmission endpoint')

args = parser.parse_args()

resolution = (args.width, args.height)
camera = PiCamera()
camera.resolution = resolution
camera.framerate = args.frame_rate
raw_capture = PiRGBArray(camera, size=resolution)
stream = camera.capture_continuous(raw_capture, format="bgr", use_video_port=True)
print('Starting camera...')
time.sleep(0.1)
with Service(width=args.width,
             height=args.height,
             endpoint=args.endpoint,
             dylibpath=args.lib_path) as svc:
    last_frame = time.time()
    sum = 0.0
    samples = 0
    for frame in stream:
        image = frame.array
        svc.send_frame(image)
        raw_capture.truncate(0)
        now = time.time()
        delta = now - last_frame
        sum += delta
        samples += 1
        last_frame = now
        if samples % 30 == 0:
            delta = sum / samples
            sum = 0.0
            samples = 0
            print(f'\rframe time: {delta} seconds ({1.0 / delta} fps)')

