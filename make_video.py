import re
from concurrent.futures import ProcessPoolExecutor

import cv2
import os

import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageEnhance

if os.path.exists("/System/Library/Fonts/Supplemental/AmericanTypewriter.ttc"):
    font_path = "/System/Library/Fonts/Supplemental/AmericanTypewriter.ttc"  # Replace with the actual path to the font file.
else:
    font_path = "../AmericanTypewriter.ttc"
font = ImageFont.truetype(font_path, size=150)
legend = Image.open("legend.png")
legend = legend.resize([int(1.35 * s) for s in legend.size])


def add_text_to_frame(frame_batch):

    frames_with_text = []
    print("Got texts", [x[1] for x in frame_batch])
    prev_frame = None
    for frame, text in frame_batch:

        # Convert frame to PIL image
        pil_image = Image.fromarray(cv2.cvtColor(frame, cv2.COLOR_BGR2RGB))

        # Load the custom font

        # Get the size of the text to place it correctly

        # Calculate the position to place the text at the bottom right corner
        text_x = frame.shape[1] - 100
        text_y = frame.shape[0] - 530

        legend_x = frame.shape[1] - legend.size[0] - 90
        legend_y = frame.shape[0] - legend.size[1] - 90

        # Add the text to the frame
        # Anchor text bottom right
        # Draw text but with bottom right anchor
        pil_image = ImageEnhance.Brightness(pil_image).enhance(1.35)

        pil_image.paste(legend, (legend_x, legend_y), legend)

        draw = ImageDraw.Draw(pil_image)

        draw.text((text_x, text_y), text, font=font, fill=(255, 255, 255), anchor = "rs")  # White color (RGB format)

        if prev_frame is not None:
            pil_image_blend = Image.blend(pil_image, prev_frame, 0.5)
            frames_with_text.append((cv2.cvtColor(np.array(pil_image_blend), cv2.COLOR_RGB2BGR), text))

        # Increase brightness of image
        # Convert back to OpenCV format
        frames_with_text.append((cv2.cvtColor(np.array(pil_image), cv2.COLOR_RGB2BGR), text))

        prev_frame = pil_image

    return frames_with_text

def process_single_image(image_paths):
    frames = [(cv2.imread(image_path[0]), image_path[1]) for image_path in image_paths]
    frames_with_text = add_text_to_frame(frames)
    return frames_with_text

def format_seconds(seconds):
    hours = seconds // 3600
    minutes = (seconds % 3600) // 60
    seconds = seconds % 60

    return f"{hours:02}:{minutes:02}:{seconds:02}"

def create_video_from_images(image_list, output_file, output_dir):
    frame_size = None

    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_file, fourcc, 24, frame_size)

    all_frames = []

    with ProcessPoolExecutor(max_workers=7) as executor:
        processed = 0
        futures = []
        WINDOW = 20
        for i in range(0, len(image_list), WINDOW):
            # text_to_add =
            # Image paths are of the form ".../test_{number}.png"
            # Extract the {number} using regex

            images = image_list[i:i+WINDOW]

            future = executor.submit(process_single_image, images)
            futures.append(future)

        for future in futures:
            frames = future.result()
            processed += len(frames)

            if processed % 10 == 0:
                print("Processed ", processed, "out of ", len(image_list), " images.")

            for frame, time in frames:
                # all_frames.append((frame, time))
                cv2.imwrite(f"{output_dir}/processed{time}.png", frame)

    all_frames = sorted(all_frames, key=lambda x: x[1])

    for frame, time in all_frames:
        if frame_size is None:
            frame_size = (frame.shape[1], frame.shape[0])
            out = cv2.VideoWriter(output_file, fourcc, 24, frame_size)

        out.write(frame)

    out.release()

import sys

# [5, 10, 15, 7] Target: 100
# solution(95) + 5
# solution(90) + 10


# ways(10) = ways(8) + ways(9)
# ways(8) = ways(6) + ways(7)


# [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
if __name__ == "__main__":
    # Replace "image_folder" with the path to the folder containing your PNG images.
    image_folder = sys.argv[1]
    output_video_file = "/Users/henry/Documents/imgs-toronto/output_video.mp4"
    output_video_file = sys.argv[2]

    image_list = [os.path.join(image_folder, img) for img in os.listdir(image_folder) if img.endswith(".png") and img.startswith("test_")]
    numbers = [format_seconds(int(re.findall(r"test_(\d+)( \(1\))?.png", image_path)[0][0])) for image_path in image_list]

    images = [x for x in zip(image_list, numbers)]
    images = sorted(images, key=lambda x: x[1])
    images = images[0:100]

    create_video_from_images(images, output_video_file, image_folder)
