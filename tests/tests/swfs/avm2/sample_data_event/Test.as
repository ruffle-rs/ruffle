package {

import flash.display.Sprite;
import flash.events.SampleDataEvent;
import flash.media.Sound;

public class Test extends Sprite {
    // Frequency of the synthesized tone, in Hz. Kept well below Nyquist so that
    // the mixer's resampler preserves the amplitude regardless of phase.
    private static const FREQUENCY:Number = 440.0;

    // Peak amplitude of the synthesized tone. The mixed audio output is
    // expected to peak at roughly this value on every frame the sound plays.
    private static const AMPLITUDE:Number = 0.5;

    // Stereo sample pairs written on each SampleDataEvent. Flash requires at
    // least 2048 pairs to keep a generated sound playing.
    private static const SAMPLES_PER_CALLBACK:int = 4096;

    // After this many sample-frames (~0.5s at 44100 Hz) the generator stops
    // providing any samples. A callback that supplies fewer than the required
    // 2048 pairs ends the generated sound, so the output falls silent.
    private static const SAMPLES_TO_GENERATE:int = 22050;

    // Running sample index, so the sine wave stays continuous across callbacks.
    private var phase:int = 0;

    public function Test() {
        var sound = new Sound();
        sound.addEventListener(SampleDataEvent.SAMPLE_DATA, onSampleData);

        // Playing a Sound that was never loaded turns it into a generated
        // sound, driven by the SAMPLE_DATA events dispatched above.
        sound.play();
    }

    private function onSampleData(event:SampleDataEvent):void {
        for (var i:int = 0; i < SAMPLES_PER_CALLBACK; i++) {
            var value:Number =
                AMPLITUDE * Math.sin(2.0 * Math.PI * FREQUENCY * phase / 44100.0);
            // Interleaved stereo: left channel then right channel.
            event.data.writeFloat(value);
            event.data.writeFloat(value);

            phase++;
            // Once half a second of audio has been generated, stop writing samples.
            // Supplying less than 2048 pairs ends the generated sound and the output goes silent.
            if (phase >= SAMPLES_TO_GENERATE) {
                break;
            }
        }
    }
}
}
