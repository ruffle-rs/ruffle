ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=8000" -af "volume=5" -ac 1 -c:a pcm_alaw tone_alaw.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=8000" -af "volume=5" -ac 1 -c:a pcm_mulaw tone_mulaw.flv
