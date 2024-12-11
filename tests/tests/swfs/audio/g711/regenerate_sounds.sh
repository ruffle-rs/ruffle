ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=8000" -af "volume=5" -ac 2 -c:a pcm_alaw tone_alaw_stereo.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=8000" -af "volume=5" -ac 1 -c:a pcm_alaw tone_alaw_mono.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=8000" -af "volume=5" -ac 2 -c:a pcm_mulaw tone_mulaw_stereo.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=8000" -af "volume=5" -ac 1 -c:a pcm_mulaw tone_mulaw_mono.flv
