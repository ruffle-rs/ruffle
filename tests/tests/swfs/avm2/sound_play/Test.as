package {
	public class Test {
	}
}

trace("///var silence = new Silence();");
var silence = new Silence();

trace("///var silence_channel = silence.play();");
var silence_channel = silence.play();

trace(silence_channel);

trace("///var noise = new Noise();");
var noise = new Noise();

trace("///var noise_channel = noise.play(2000);");
var noise_channel = noise.play(2000);

trace(noise_channel);

trace("///noise_channel = noise.play(1800);");
noise_channel = noise.play(1800);

trace(noise_channel);

trace("///noise_channel = noise.play(1600);");
noise_channel = noise.play(1600);

trace(noise_channel);

trace("///noise_channel = noise.play(1400);");
noise_channel = noise.play(1400);

trace(noise_channel);

trace("///noise_channel = noise.play(1200);");
noise_channel = noise.play(1200);

trace(noise_channel);

trace("///noise_channel = noise.play(1000);");
noise_channel = noise.play(1000);

trace(noise_channel);

trace("///var lofi_silence = new LofiSilence();");
var lofi_silence = new LofiSilence();

trace("///var lofi_silence_channel = lofi_silence.play(700);");
var lofi_silence_channel = lofi_silence.play(700);

trace(lofi_silence_channel);