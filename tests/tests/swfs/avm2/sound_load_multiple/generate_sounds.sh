#!/bin/bash

ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=44100" -af "volume=5" -ac 1 -c:a mp3 sound.mp3
ffmpeg -y -i sound.mp3 -f f32le -c:a pcm_f32le sound.pcm
