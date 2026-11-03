// How many `sampleData` events Flash Player dispatches before `play` returns,
// the positions it reports for a `play` with a start time, and `load` after
// `play` of a sound without data. The output is Flash Player 32's.
package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.SampleDataEvent;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.net.URLRequest;
    import flash.text.TextField;
    import flash.utils.setTimeout;

    public class Test extends Sprite {
        private var tf:TextField = new TextField();
        private var steps:Array;

        private function log(s:String):void {
            trace(s);
            tf.appendText(s + "\n");
        }

        public function Test() {
            tf.width = 800;
            tf.height = 1000;
            tf.multiline = true;
            addChild(tf);
            steps = [
                // In Flash, the first sound waits for the audio device to
                // start, so this one is played twice.
                function():void { blocks([4096]); },
                function():void { blocks([2048]); },
                function():void { blocks([2049]); },
                function():void { blocks([3000]); },
                function():void { blocks([4096]); },
                function():void { blocks([6000]); },
                function():void { blocks([8192]); },
                function():void { blocks([8192, 2048]); },
                function():void { blocks([2048, 8192]); },
                function():void { blocks([2049, 8192]); },
                function():void { blocks([3000, 2048]); },
                function():void { blocks([4096, 2048]); },
                function():void { blocks([4096, 4096, 2833, 4096]); },
                function():void { playStart(500, 3); },
                function():void { playStart(1000, 1); },
                function():void { playStart(0, 3); },
                playingPosition,
                loadAfterPlay
            ];
            next();
        }

        private function next():void {
            if (steps.length == 0) {
                log("done");
                return;
            }
            steps.shift()();
        }

        // The listener writes the block sizes in `sizes`, repeating the last.
        private function blocks(sizes:Array):void {
            var events:int = 0;
            var queued:int = 0;
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                var samples:int = sizes[Math.min(events, sizes.length - 1)];
                events++;
                queued += samples;
                for (var i:int = 0; i < samples; i++) {
                    e.data.writeFloat(0);
                    e.data.writeFloat(0);
                }
            });
            var channel:SoundChannel = sound.play();
            log(sizes.join(",") + ": " + events + " events before play() returned, " + queued + " samples");
            channel.stop();
            setTimeout(next, 100);
        }

        private function playStart(start:Number, loops:int):void {
            var positions:Array = [];
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                positions.push(e.position);
                var samples:int = positions.length <= 3 ? 4096 : 1024;
                for (var i:int = 0; i < samples; i++) {
                    e.data.writeFloat(0);
                    e.data.writeFloat(0);
                }
            });
            var channel:SoundChannel = sound.play(start, loops);
            positions.push("returned");
            channel.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                log("play(" + start + ", " + loops + "): positions " + positions.join(" ") +
                    ", soundComplete at " + channel.position.toFixed(2));
                setTimeout(next, 100);
            });
        }

        private function playingPosition():void {
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                for (var i:int = 0; i < 4096; i++) {
                    e.data.writeFloat(0);
                    e.data.writeFloat(0);
                }
            });
            var channel:SoundChannel = sound.play(1000);
            log("play(1000): position after play() " + channel.position.toFixed(2));
            setTimeout(function():void {
                var position:Number = channel.position;
                log("play(1000): position after 500 ms between 1000 and 2000 " + (position >= 1000 && position < 2000) +
                    ", below 1000 " + (position < 1000));
                channel.stop();
                setTimeout(next, 100);
            }, 500);
        }

        private function loadAfterPlay():void {
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                for (var i:int = 0; i < 4096; i++) {
                    e.data.writeFloat(0);
                    e.data.writeFloat(0);
                }
            });
            var first:SoundChannel = sound.play();
            var second:SoundChannel = sound.play();
            log("played twice: two channels " + (first != null && second != null && first != second));
            try {
                sound.load(new URLRequest("missing.mp3"));
                log("load: no error");
            } catch (e:Error) {
                log("load: " + e.errorID);
            }
            first.stop();
            second.stop();
            setTimeout(next, 100);
        }
    }
}
