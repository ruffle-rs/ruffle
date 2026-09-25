package {

import flash.display.Sprite;
import flash.events.SampleDataEvent;
import flash.media.Sound;

public class Test extends Sprite {
    private static const SAMPLES_PER_CALLBACK:int = 4096;
    private static const SAMPLES_TO_GENERATE:int = 22050;

    private var phase:int = 0;

    public function Test() {
        var sound = new Sound();
        sound.addEventListener(SampleDataEvent.SAMPLE_DATA, onSampleData);
        sound.play();
    }

    private function onSampleData(event:SampleDataEvent):void {
        trace("SampleDataEvent received");
        trace("Phase: " + phase);

        for (var i:int = 0; i < SAMPLES_PER_CALLBACK; i++) {
            event.data.writeFloat(0.0);
            event.data.writeFloat(0.0);

            phase++;

            if (phase >= SAMPLES_TO_GENERATE / 2) {
                trace("Setting event.data to null at phase: " + phase);
                event.data = null;
            }

            if (phase >= SAMPLES_TO_GENERATE) {
                trace("Stopping sound generation at phase: " + phase);
                break;
            }
        }
    }
}
}
