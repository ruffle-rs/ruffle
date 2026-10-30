package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Sound;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    // stereo.mp3: 1 s of a 441 Hz sine at 0.5, the right channel inverted,
    // 44.1 kHz, LAME 128 kbps with an info frame (encoder delay 576).
    // mono22.mp3: 0.5 s of the same sine, 22.05 kHz mono, LAME 64 kbps.
    // noise.mp3: a few frames, without an info frame.
    public class Test extends Sprite {
        public function Test() {
            trace("no data: extract() returned " + new Sound().extract(new ByteArray(), 1000, 0));
            load("stereo.mp3", function(sound:Sound):void {
                testStereo(sound);
                load("mono22.mp3", function(sound:Sound):void {
                    testMono(sound);
                    load("noise.mp3", testShort);
                });
            });
        }

        private static function load(url:String, then:Function):void {
            var sound:Sound = new Sound();
            sound.addEventListener(Event.COMPLETE, function(e:Event):void { then(sound); });
            sound.load(new URLRequest(url));
        }

        private static function sample(bytes:ByteArray, index:int):String {
            bytes.position = index * 8;
            var left:Number = bytes.readFloat();
            var right:Number = bytes.readFloat();
            return left.toFixed(2) + "/" + right.toFixed(2);
        }

        private static function firstAudible(bytes:ByteArray, count:int):int {
            for (var i:int = 0; i < count; i++) {
                bytes.position = i * 8;
                if (Math.abs(bytes.readFloat()) > 0.05) {
                    return i;
                }
            }
            return -1;
        }

        private static function same(a:ByteArray, aIndex:int, b:ByteArray, bIndex:int, count:int):Boolean {
            for (var i:int = 0; i < count * 2; i++) {
                a.position = aIndex * 8 + i * 4;
                b.position = bIndex * 8 + i * 4;
                if (Math.abs(a.readFloat() - b.readFloat()) > 0.001) {
                    return false;
                }
            }
            return true;
        }

        private static function testStereo(sound:Sound):void {
            // The length and the samples include the info frame, decoded as
            // 1152 samples of silence, then the encoder and decoder delays.
            trace("stereo: length " + sound.length);

            // Without startPosition, extract() goes on from where the last call ended.
            var sequential:ByteArray = new ByteArray();
            var first:Number = sound.extract(sequential, 1000);
            var second:Number = sound.extract(sequential, 1000);
            var explicit:ByteArray = new ByteArray();
            var third:Number = sound.extract(explicit, 500, 2300);
            var fourth:Number = sound.extract(sequential, 1000);
            var fifth:Number = sound.extract(sequential, 1000, -5);
            trace("stereo: sequential calls returned " + first + ", " + second + ", " + fourth + ", " + fifth +
                ", explicit call " + third + ", target length " + sequential.length);

            var all:ByteArray = new ByteArray();
            var count:Number = sound.extract(all, 1000000, 0);
            trace("stereo: extract(all) returned " + count + ", length " + all.length + ", position " + all.position);
            trace("stereo: first sample above 0.05 at " + firstAudible(all, count));
            trace("stereo: samples " + sample(all, 2300) + ", " + sample(all, 2325) + ", " + sample(all, 3000));

            trace("stereo: first two calls match extract(all) " + same(sequential, 0, all, 0, 2000));
            trace("stereo: explicit call matches " + same(explicit, 0, all, 2300, 500));
            trace("stereo: call after the explicit one goes on from 2000 " + same(sequential, 2000, all, 2000, 1000) +
                ", from 2800 " + same(sequential, 2000, all, 2800, 1000));
            trace("stereo: negative startPosition goes on from 3800 " + same(sequential, 3000, all, 3800, 1000));

            var bytes:ByteArray = new ByteArray();
            trace("stereo: 100000 from the last frame returned " + sound.extract(bytes, 100000, count - 1152));
            trace("stereo: 100 from past the end returned " + sound.extract(bytes, 100, count + 10));
            // How lengths and positions out of range convert, by the samples left from there.
            var starts:Array = [-5, -Infinity, -2147483649, 1600.9, 2147483647, 2147483648, 1e12, Infinity, NaN];
            var lefts:Array = [];
            for each (var startPosition:Number in starts) {
                sound.extract(bytes, 3, 40000);
                lefts.push(sound.extract(bytes, 100000, startPosition));
            }
            trace("stereo: samples left from 40003 after start " + starts.join(", ") + ": " + lefts.join(", "));
            trace("stereo: length 2^32+3 returned " + sound.extract(bytes, 4294967299, 0) + ", Infinity " +
                sound.extract(bytes, Infinity, 0) + ", NaN " + sound.extract(bytes, NaN, 0));
            trace("stereo: length 0 returned " + sound.extract(bytes, 0, 0));
            trace("stereo: length -5 returned " + sound.extract(bytes, -5, 0));
            trace("stereo: length 10.7 returned " + sound.extract(bytes, 10.7, 0));
            var fractional:ByteArray = new ByteArray();
            sound.extract(fractional, 10, 2300.6);
            trace("stereo: startPosition 2300.6 matches 2300 " + same(fractional, 0, all, 2300, 10));

            // Samples are written at the target's position, with its endianness.
            var target:ByteArray = new ByteArray();
            target.writeInt(7);
            target.position = 2;
            target.endian = Endian.LITTLE_ENDIAN;
            var written:Number = sound.extract(target, 10, 2300);
            trace("stereo: wrote " + written + " at position 2, length " + target.length + ", position " + target.position);
            target.position = 2;
            trace("stereo: little-endian sample " + target.readFloat().toFixed(2));
        }

        private static function testMono(sound:Sound):void {
            // Resampled to 44.1 kHz stereo.
            trace("mono22: length " + sound.length);
            var all:ByteArray = new ByteArray();
            var count:Number = sound.extract(all, 1000000, 0);
            trace("mono22: extract(all) returned " + count + ", length " + all.length);
            var equal:Boolean = true;
            for (var i:int = 0; i < count; i++) {
                all.position = i * 8;
                if (all.readFloat() != all.readFloat()) {
                    equal = false;
                }
            }
            trace("mono22: both channels equal " + equal);
        }

        private static function testShort(sound:Sound):void {
            trace("noise: extracted samples " + (sound.extract(new ByteArray(), 1000000, 0) > 0));
            trace("done");
        }
    }
}
