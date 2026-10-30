package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.SampleDataEvent;
    import flash.media.Sound;
    import flash.media.SoundTransform;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    public class Test extends Sprite {
        private var frame:int = 0;

        public function Test() {
            // Little-endian samples are read as little-endian.
            play(Endian.LITTLE_ENDIAN, 8, null);
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                frame++;
                if (frame == 15) {
                    play(Endian.BIG_ENDIAN, 4, new SoundTransform(0.5));
                }
            });
        }

        // Plays a 441 Hz sine at amplitude 0.5 for `blocks` blocks of 2048 samples.
        private static function play(endian:String, blocks:int, transform:SoundTransform):void {
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                if (e.position >= blocks * 2048) {
                    return;
                }
                e.data.endian = endian;
                for (var i:int = 0; i < 2048; i++) {
                    var sample:Number = 0.5 * Math.sin((e.position + i) * 2 * Math.PI / 100);
                    e.data.writeFloat(sample);
                    e.data.writeFloat(sample);
                }
            });
            sound.play(0, 0, transform);
        }
    }
}
