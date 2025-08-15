ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=44100" -af "volume=5" -ac 2 -c:a aac tone_stereo_44100hz.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=48000" -af "volume=5" -ac 2 -c:a aac tone_stereo_48000hz.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=22050" -af "volume=5" -ac 2 -c:a aac tone_stereo_22050hz.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=44100" -af "volume=5" -ac 1 -c:a aac tone_mono_44100hz.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=48000" -af "volume=5" -ac 1 -c:a aac tone_mono_48000hz.flv
ffmpeg -y -f lavfi -i "sine=frequency=444:duration=1:sample_rate=22050" -af "volume=5" -ac 1 -c:a aac tone_mono_22050hz.flv