package {

import flash.display.Sprite;
import flash.events.SampleDataEvent;
import flash.media.Sound;
import flash.media.SoundChannel;

public class Test extends Sprite {

    public function Test() {
        for (var i:int = 0; i < 100; i++) {
            var s:Sound = new Sound();
            s.addEventListener(SampleDataEvent.SAMPLE_DATA, onSampleData);
            var ch:SoundChannel = s.play();
            trace(i, ch);
        }
    }

    private function onSampleData(event:SampleDataEvent):void {
        for (var i:int = 0; i < 4096; i++) {
            event.data.writeFloat(0.0);
            event.data.writeFloat(0.0);
        }
    }
}
}
