// Compiled with Apache Flex (for the embedded sound):
// mxmlc -static-link-runtime-shared-libraries -omit-trace-statements=false -swf-version=15 -target-player=11.2 -default-frame-rate=30 Test.as -o test.swf
package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Sound;
    import flash.media.SoundChannel;

    // SoundChannel.position after a sound has finished playing: the length of
    // the sound, whether or not the position was read before. Only stopping a
    // sound before reading its position leaves the position at 0.
    public class Test extends Sprite {
        [Embed(source="silence.mp3")]
        private static const Silence:Class;

        private var frames:int = 0;
        private var channels:Object = {};
        private var atComplete:Object = {};

        public function Test() {
            var sound:Sound = new Silence();
            trace("sound.length: " + sound.length);

            play(sound, "never read", 0, 0);
            play(sound, "read at the start", 0, 0);
            // Read the position (its exact value right after starting depends on
            // the audio output), so that it's cached.
            var position:Number = channels["read at the start"].position;
            play(sound, "looped twice", 0, 2);
            play(sound, "started at 300", 300, 0);

            var stopped:SoundChannel = sound.play();
            stopped.stop();
            channels["stopped before reading"] = stopped;

            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function play(sound:Sound, name:String, start:Number, loops:int):void {
            var channel:SoundChannel = sound.play(start, loops);
            channels[name] = channel;
            channel.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                atComplete[name] = channel.position;
            });
        }

        private function onEnterFrame(e:Event):void {
            frames++;
            if (frames != 120) {
                return;
            }

            removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            for each (var name:String in ["never read", "read at the start", "looped twice", "started at 300"]) {
                trace(name + ": at soundComplete " + atComplete[name] + ", later " + channels[name].position);
            }
            trace("stopped before reading: " + channels["stopped before reading"].position);
        }
    }
}
