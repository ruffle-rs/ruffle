package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.events.SampleDataEvent;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        public function Test() {
            playBasic();
        }

        private static function writeSamples(data:ByteArray, count:int):void {
            for (var i:int = 0; i < count; i++) {
                data.writeFloat(0.25);
                data.writeFloat(-0.25);
            }
        }

        private static function onComplete(channel:SoundChannel, name:String, next:Function):void {
            channel.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                trace(name + ": soundComplete, channel.position " + channel.position.toFixed(2));
                next();
            });
        }

        // Asked for samples while it plays; a short block ends it.
        private function playBasic():void {
            var sound:Sound = new Sound();
            var requests:int = 0;
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                requests++;
                trace("basic: sampleData " + requests + ", position " + e.position +
                    ", data.length " + e.data.length + ", data.position " + e.data.position +
                    ", " + e.data.endian + ", target is the sound " + (e.target == sound));
                writeSamples(e.data, requests < 6 ? 2048 : 1000);
            });
            trace("basic: play()");
            var channel:SoundChannel = sound.play();
            trace("basic: play() returned " + channel + ", length " + sound.length +
                ", bytesTotal " + sound.bytesTotal);
            onComplete(channel, "basic", function():void { replay(sound); });
        }

        // Playing the same sound again starts again from position 0.
        private function replay(sound:Sound):void {
            trace("replay: play()");
            onComplete(sound.play(), "replay", playEmpty);
        }

        // No samples at all end the sound right away.
        private function playEmpty():void {
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                trace("empty: sampleData, position " + e.position);
            });
            onComplete(sound.play(), "empty", playRewound);
        }

        // Only the samples before data.position count.
        private function playRewound():void {
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                trace("rewound: sampleData, position " + e.position);
                writeSamples(e.data, 4096);
                e.data.position = 1024 * 8;
            });
            onComplete(sound.play(), "rewound (1024 samples)", playOversized);
        }

        // More than 8192 samples are not played, and end the sound.
        private function playOversized():void {
            var sound:Sound = new Sound();
            sound.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                trace("oversized: sampleData, position " + e.position);
                writeSamples(e.data, 8193);
            });
            onComplete(sound.play(), "oversized (8193 samples)", playRest);
        }

        private function playRest():void {
            // Flash keeps asking while it gets blocks of 8192 samples, up to five.
            var big:Sound = new Sound();
            var inPlay:Boolean = true;
            big.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                trace("big blocks: sampleData, position " + e.position + ", in play() " + inPlay);
                writeSamples(e.data, 8192);
            });
            var bigChannel:SoundChannel = big.play();
            inPlay = false;
            bigChannel.stop();

            // Every channel asks for its own samples.
            var two:Sound = new Sound();
            var channel:String = "first";
            two.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                trace("two channels: sampleData for the " + channel + " channel, position " + e.position);
                writeSamples(e.data, 2048);
            });
            var first:SoundChannel = two.play();
            channel = "second";
            var second:SoundChannel = two.play();
            first.stop();
            second.stop();

            // A sound with no data and no listener is invalid.
            try {
                new Sound().play();
                trace("no listener: play() did not throw");
            } catch (e:Error) {
                trace("no listener: play() threw " + getQualifiedClassName(e) + " " + e.errorID);
            }

            // A sound that loads plays what it loads, not what listeners generate.
            var loading:Sound = new Sound();
            loading.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                trace("loading: sampleData");
            });
            loading.addEventListener(IOErrorEvent.IO_ERROR, function(e:IOErrorEvent):void {
                trace("loading: ioError");
                playStopped();
            });
            loading.load(new URLRequest("missing.mp3"));
            trace("loading: play() returned " + loading.play());
        }

        // Stopped before its last samples have played, a sound does not
        // jump to their end.
        private function playStopped():void {
            var stopped:Sound = new Sound();
            var requests:int = 0;
            stopped.addEventListener(SampleDataEvent.SAMPLE_DATA, function(e:SampleDataEvent):void {
                requests++;
                writeSamples(e.data, requests == 1 ? 2048 : 1000);
            });
            var stoppedChannel:SoundChannel = stopped.play();
            var frames:int = 0;
            addEventListener(Event.ENTER_FRAME, function stop(e:Event):void {
                if (++frames < 2) {
                    return;
                }
                removeEventListener(Event.ENTER_FRAME, stop);
                stoppedChannel.stop();
                trace("stopped: position before the end " +
                    (stoppedChannel.position < 3048 / 44.1));
            });
        }
    }
}
