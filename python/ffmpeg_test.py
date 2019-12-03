import ffmpeg
import numpy as np
from subprocess import Popen, PIPE

def capture_frames(path, frame_callback):
    print("[ffmpeg] capture_frames started!")    
    stream = (
        ffmpeg
        .input(path, format="webm")
        .filter("fps", "60")
        .output("pipe:1", format="rawvideo", pix_fmt="rgb24", s="112x112")
    )
    args = stream.compile()
    # print(f"args={' '.join(args)}")

    # "-loglevel", "0"
    process = Popen(["ffmpeg", "-hide_banner", "-nostats"] + args[1:], stdout=PIPE)
    
    while True:
        in_bytes = process.stdout.read(112 * 112 * 3)
        if not in_bytes:
            break
        # frame = np.frombuffer(in_bytes, np.uint8).reshape([112, 112, 3])
        frame_callback(in_bytes)
    process.wait()
    print("[ffmpeg] capture_frames completed!")


if __name__ == "__main__":
    sample_path = "/Users/aslam/Downloads/ffmpeg_samples/Schlossbergbahn.webm.480p.vp9.webm"
    count = 0
    def callback_frame(frame):
        global count
        count += 1
        # print(f"frame={frame.shape}")
    capture_frames(sample_path, callback_frame)
    print(f"total_frames={count}")