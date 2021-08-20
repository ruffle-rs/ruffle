package {
	public class Test {
	}
}

import EventWatcher;

trace("///var silence = new Silence();");
var silence = new Silence();

trace("///var silence_channel = silence.play();");
var silence_channel = silence.play();

trace("///var ew = new EventWatcher(\"silence_channel\", silence_channel);");
var ew = new EventWatcher("silence_channel", silence_channel);

trace("///var noise = new Noise();");
var noise = new Noise();

trace("///var noise_channel = noise.play(1000);");
var noise_channel = noise.play(1000);

trace("///var ew2 = new EventWatcher(\"noise_channel\", noise_channel);");
var ew2 = new EventWatcher("noise_channel", noise_channel);