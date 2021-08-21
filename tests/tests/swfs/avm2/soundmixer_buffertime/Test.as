package {
	public class Test {
	}
}

import flash.media.SoundMixer;

trace("///SoundMixer.bufferTime");
trace(SoundMixer.bufferTime);

trace("///SoundMixer.bufferTime = 120");
SoundMixer.bufferTime = 120;

trace("///SoundMixer.bufferTime");
trace(SoundMixer.bufferTime);