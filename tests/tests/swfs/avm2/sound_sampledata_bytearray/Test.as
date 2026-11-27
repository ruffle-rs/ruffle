package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.SampleDataEvent;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.utils.ByteArray;

    // Which `ByteArray` the `sampleData` listeners of a sound write into,
    // across the channels playing it.
    public class Test extends Sprite {
        private var sound:Sound = new Sound();
        private var label:String;
        private var firstData:ByteArray;

        public function Test() {
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, onSampleData);
            playTwice();
        }

        private static function writeSamples(data:ByteArray, count:int):void {
            for (var i:int = 0; i < count; i++) {
                data.writeFloat(0.25);
                data.writeFloat(-0.25);
            }
        }

        // Fewer than 2048 samples end each channel after its first request,
        // so the requests don't depend on timing.
        private function onSampleData(e:SampleDataEvent):void {
            if (firstData == null) {
                firstData = e.data;
            }
            trace(label + ": sampleData, position " + e.position +
                ", data.length " + e.data.length + ", data.position " + e.data.position +
                ", same data as the first channel " + (e.data === firstData));
            writeSamples(e.data, label == "first channel" ? 1000 : 500);
        }

        // Two channels playing the same sound at the same time.
        private function playTwice():void {
            label = "first channel";
            sound.play();
            label = "second channel";
            var second:SoundChannel = sound.play();
            second.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                trace("second channel: soundComplete");
                playAgain();
            });
        }

        // A channel playing the same sound after the others have ended.
        private function playAgain():void {
            label = "later channel";
            sound.play().addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                trace("later channel: soundComplete");
                playOther();
            });
        }

        // A channel playing a different sound.
        private function playOther():void {
            var other:Sound = new Sound();
            other.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                trace("other sound: sampleData, position " + e.position +
                    ", data.length " + e.data.length + ", data.position " + e.data.position +
                    ", same data as the first channel " + (e.data === firstData));
                writeSamples(e.data, 500);
            });
            other.play().addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                trace("other sound: soundComplete");
                trace("done");
            });
        }
    }
}
