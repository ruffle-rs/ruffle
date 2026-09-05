package {

import flash.display.Sprite;
import flash.events.SampleDataEvent;
import flash.media.Sound;
import flash.media.SoundChannel;
import flash.net.URLRequest;

public class Test extends Sprite {

    public function Test() {
        var s:Sound = new Sound();
        s.addEventListener(SampleDataEvent.SAMPLE_DATA, onSampleData);
        var ch2:SoundChannel = s.play();
        var ch1:SoundChannel = s.play();
        trace(ch1 == ch2);
        try {
            // The problem is not that missing.mp3 doesn't exist, but that this
            // sound is now procedurally generated, so nothing can be loaded into it.
            s.load(new URLRequest("missing.mp3"));
            trace("Load succeeded unexpectedly");
        } catch (error:Error) {
            trace("An error occurred: " + error.message);
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
