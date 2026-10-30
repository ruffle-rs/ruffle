package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.media.SoundTransform;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;

    // info.mp3 and noinfo.mp3 are the same 1 s of noise, encoded with and
    // without a LAME info frame. Flash Player decodes the info frame as a
    // frame of silence (1152 sample frames): it counts in Sound.length, and
    // played together (panned apart, and recorded), info.mp3 sounds 1152
    // sample frames after noinfo.mp3, and that much later again on every loop.
    // badtag.mp3 is info.mp3 with a non-zero byte in the info frame's side
    // information, which the decoder doesn't take for an info frame; Flash
    // plays it like info.mp3. noframes.mp3 is info.mp3 with an info frame that
    // doesn't give the number of frames; Flash counts them.
    [SWF(frameRate="100")]
    public class Test extends Sprite {
        public function Test() {
            var info:Sound = new Sound(new URLRequest("info.mp3"));
            info.addEventListener(Event.COMPLETE, function(e:Event):void {
                var noinfo:Sound = new Sound(new URLRequest("noinfo.mp3"));
                noinfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                    var badtag:Sound = new Sound(new URLRequest("badtag.mp3"));
                    badtag.addEventListener(Event.COMPLETE, function(e:Event):void {
                        var noframes:Sound = new Sound(new URLRequest("noframes.mp3"));
                        noframes.addEventListener(Event.COMPLETE, function(e:Event):void {
                            trace("length badtag " + badtag.length + ", extracted " +
                                badtag.extract(new ByteArray(), 1000000, 0) + ", info extracted " +
                                info.extract(new ByteArray(), 1000000, 0));
                            trace("length noframes " + noframes.length + ", extracted " +
                                noframes.extract(new ByteArray(), 1000000, 0));
                            run(info, noinfo, noframes);
                        });
                    });
                });
            });
        }

        private function run(info:Sound, noinfo:Sound, noframes:Sound):void {
            trace("length info " + info.length + ", noinfo " + noinfo.length);
            var a:SoundChannel = info.play(0, 0, new SoundTransform(1, -1));
            var b:SoundChannel = noinfo.play(0, 0, new SoundTransform(1, 1));
            a.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void { trace("info complete"); });
            b.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                trace("noinfo complete");
                // noframes.mp3 plays to the end, like info.mp3.
                var c:SoundChannel = noframes.play();
                var d:SoundChannel = noinfo.play();
                c.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void { trace("noframes complete"); });
                d.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void { trace("noinfo complete again"); });
            });
        }
    }
}
