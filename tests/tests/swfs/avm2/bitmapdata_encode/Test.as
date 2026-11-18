package {
    import flash.display.BitmapData;
    import flash.display.MovieClip;
    import flash.display.PNGEncoderOptions;
    import flash.display.JPEGEncoderOptions;
    import flash.geom.Rectangle;
    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.utils.getQualifiedClassName;

    public class Test extends MovieClip {
        public function Test() {
            var bd:BitmapData = makeBd(20, 20, true);
            var opaque:BitmapData = makeBd(20, 20, false);

            section("PNG default, full rect, transparent bd");
            describePng(bd.encode(new Rectangle(0, 0, 20, 20), new PNGEncoderOptions()));

            section("PNG fastCompression=true, full rect, transparent bd");
            describePng(bd.encode(new Rectangle(0, 0, 20, 20), new PNGEncoderOptions(true)));

            section("PNG default, subrect (2,3,5,7), transparent bd");
            describePng(bd.encode(new Rectangle(2, 3, 5, 7), new PNGEncoderOptions()));

            section("PNG default, full rect, opaque bd");
            describePng(opaque.encode(new Rectangle(0, 0, 20, 20), new PNGEncoderOptions()));

            section("JPEG quality=80, full rect, opaque bd");
            describeJpeg(opaque.encode(new Rectangle(0, 0, 20, 20), new JPEGEncoderOptions()));

            section("JPEG quality=1, full rect, opaque bd");
            describeJpeg(opaque.encode(new Rectangle(0, 0, 20, 20), new JPEGEncoderOptions(1)));

            section("JPEG quality=100, full rect, opaque bd");
            describeJpeg(opaque.encode(new Rectangle(0, 0, 20, 20), new JPEGEncoderOptions(100)));

            section("JPEG quality=80, subrect (2,3,5,7)");
            describeJpeg(opaque.encode(new Rectangle(2, 3, 5, 7), new JPEGEncoderOptions(80)));

            section("Default output ByteArray (third arg omitted)");
            var noArg:ByteArray = bd.encode(new Rectangle(0, 0, 4, 4), new PNGEncoderOptions());
            trace("class: " + getClassName(noArg));
            trace("length > 0: " + (noArg.length > 0));
            trace("endian: " + noArg.endian);

            section("Provided output ByteArray, returned === provided");
            var provided:ByteArray = new ByteArray();
            var ret:ByteArray = bd.encode(new Rectangle(0, 0, 4, 4), new PNGEncoderOptions(), provided);
            trace("ret === provided: " + (ret === provided));
            trace("length > 0: " + (provided.length > 0));

            section("Provided output ByteArray with existing data is overwritten from position 0");
            var pre:ByteArray = new ByteArray();
            pre.writeByte(0xAA);
            pre.writeByte(0xBB);
            pre.writeByte(0xCC);
            pre.position = 0;
            bd.encode(new Rectangle(0, 0, 4, 4), new PNGEncoderOptions(), pre);
            trace("length > 0: " + (pre.length > 0));
            pre.position = 0;
            trace("byte[0..2]: " + pre.readUnsignedByte() + "," + pre.readUnsignedByte() + "," + pre.readUnsignedByte());

            section("Provided output ByteArray, position past existing data preserves earlier bytes");
            var midBa:ByteArray = new ByteArray();
            midBa.writeByte(0x11);
            midBa.writeByte(0x22);
            midBa.writeByte(0x33);
            midBa.position = 1;
            bd.encode(new Rectangle(0, 0, 4, 4), new PNGEncoderOptions(), midBa);
            midBa.position = 0;
            trace("byte[0]: " + midBa.readUnsignedByte());

            section("Zero-area rect: width=0");
            try {
                bd.encode(new Rectangle(0, 0, 0, 5), new PNGEncoderOptions());
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Zero-area rect: height=0");
            try {
                bd.encode(new Rectangle(0, 0, 5, 0), new PNGEncoderOptions());
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Negative width rect");
            try {
                bd.encode(new Rectangle(0, 0, -5, 5), new PNGEncoderOptions());
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Negative width/height with positive origin");
            for each (var negRect:Rectangle in [new Rectangle(10, 0, -5, 5), new Rectangle(0, 10, 5, -5), new Rectangle(25, 0, -10, 5)]) {
                try {
                    bd.encode(negRect, new PNGEncoderOptions());
                    trace(negRect + ": did not throw");
                } catch (e:Error) {
                    trace(negRect + ": threw " + getQualifiedClassName(e) + " #" + e.errorID);
                }
            }

            section("Out-of-bounds rect (partially overlapping)");
            try {
                var ob:ByteArray = bd.encode(new Rectangle(15, 15, 20, 20), new PNGEncoderOptions());
                trace("length > 0: " + (ob.length > 0));
                describePngDims(ob);
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Rect fully outside bounds");
            try {
                bd.encode(new Rectangle(100, 100, 10, 10), new PNGEncoderOptions());
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Negative origin rect");
            try {
                var neg:ByteArray = bd.encode(new Rectangle(-5, -5, 10, 10), new PNGEncoderOptions());
                trace("length > 0: " + (neg.length > 0));
                describePngDims(neg);
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Null rect");
            try {
                bd.encode(null, new PNGEncoderOptions());
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Null compressor");
            try {
                bd.encode(new Rectangle(0, 0, 4, 4), null);
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Deterministic: same input -> same length");
            var a:ByteArray = bd.encode(new Rectangle(0, 0, 20, 20), new PNGEncoderOptions());
            var b:ByteArray = bd.encode(new Rectangle(0, 0, 20, 20), new PNGEncoderOptions());
            trace("same length: " + (a.length == b.length));

            section("PNG color type");
            var allOpaque:BitmapData = new BitmapData(8, 8, true, 0xFF0000FF);
            describePngDims(allOpaque.encode(allOpaque.rect, new PNGEncoderOptions()));
            allOpaque.setPixel32(7, 7, 0x00000000);
            describePngDims(allOpaque.encode(new Rectangle(0, 0, 4, 4), new PNGEncoderOptions()));
            describePngDims(allOpaque.encode(allOpaque.rect, new PNGEncoderOptions()));
            allOpaque.setPixel32(7, 7, 0xFEFF0000);
            describePngDims(allOpaque.encode(allOpaque.rect, new PNGEncoderOptions()));

            section("JPEG quality bounds");
            for each (var q:uint in [0, 100, 101, uint(-1)]) {
                try {
                    var qa:ByteArray = opaque.encode(new Rectangle(0, 0, 4, 4), new JPEGEncoderOptions(q));
                    trace("quality " + q + ": length > 0: " + (qa.length > 0));
                } catch (e:Error) {
                    trace("quality " + q + ": threw " + getQualifiedClassName(e) + " #" + e.errorID);
                }
            }

            section("Compressor of the wrong type");
            try {
                bd.encode(new Rectangle(0, 0, 4, 4), {});
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }

            section("Large dimensions");
            for each (var size:int in [65500, 65501]) {
                var wide:BitmapData = new BitmapData(size, 1, false, 0xFF00FF00);
                var tall:BitmapData = new BitmapData(1, size, false, 0xFF00FF00);
                for each (var big:BitmapData in [wide, tall]) {
                    var dims:String = big.width + "x" + big.height;
                    try {
                        big.encode(big.rect, new JPEGEncoderOptions());
                        trace(dims + " JPEG: did not throw");
                    } catch (e:Error) {
                        trace(dims + " JPEG: threw " + getQualifiedClassName(e) + " #" + e.errorID);
                    }
                    try {
                        describePngDims(big.encode(big.rect, new PNGEncoderOptions()));
                    } catch (e:Error) {
                        trace(dims + " PNG: threw " + getQualifiedClassName(e) + " #" + e.errorID);
                    }
                }
            }

            section("Error precedence");
            var disposedBd:BitmapData = new BitmapData(10, 10, false, 0);
            disposedBd.dispose();
            var hugeBd:BitmapData = new BitmapData(65501, 1, false, 0);
            var cases:Array = [
                ["disposed + null rect", disposedBd, null, new PNGEncoderOptions()],
                ["disposed + null compressor", disposedBd, new Rectangle(0, 0, 5, 5), null],
                ["disposed + invalid compressor", disposedBd, new Rectangle(0, 0, 5, 5), {}],
                ["disposed + negative rect", disposedBd, new Rectangle(10, 0, -5, 5), new PNGEncoderOptions()],
                ["disposed + quality 101", disposedBd, new Rectangle(0, 0, 5, 5), new JPEGEncoderOptions(101)],
                ["null rect + quality 101", bd, null, new JPEGEncoderOptions(101)],
                ["null rect + null compressor", bd, null, null],
                ["null rect + invalid compressor", bd, null, {}],
                ["negative rect + null compressor", bd, new Rectangle(10, 0, -5, 5), null],
                ["negative rect + invalid compressor", bd, new Rectangle(10, 0, -5, 5), {}],
                ["negative rect + quality 101", bd, new Rectangle(10, 0, -5, 5), new JPEGEncoderOptions(101)],
                ["outside rect + invalid compressor", bd, new Rectangle(100, 100, 5, 5), {}],
                ["outside rect + quality 101", bd, new Rectangle(100, 100, 5, 5), new JPEGEncoderOptions(101)],
                ["too large for JPEG + quality 101", hugeBd, hugeBd.rect, new JPEGEncoderOptions(101)],
                ["too large for JPEG + negative rect", hugeBd, new Rectangle(70000, 0, -65501, 1), new JPEGEncoderOptions()],
                ["too large for JPEG + outside rect", hugeBd, new Rectangle(70000, 0, 5, 1), new JPEGEncoderOptions()]
            ];
            for each (var c:Array in cases) {
                try {
                    BitmapData(c[1]).encode(c[2], c[3]);
                    trace(c[0] + ": did not throw");
                } catch (e:Error) {
                    trace(c[0] + ": threw " + getQualifiedClassName(e) + " #" + e.errorID);
                }
            }

            section("Disposed bitmap");
            var disposed:BitmapData = new BitmapData(10, 10, true, 0xFF000000);
            disposed.dispose();
            try {
                disposed.encode(new Rectangle(0, 0, 10, 10), new PNGEncoderOptions());
                trace("did not throw");
            } catch (e:Error) {
                trace("threw " + getQualifiedClassName(e) + " #" + e.errorID);
            }
        }

        static function section(name:String):void {
            trace("");
            trace("// " + name);
        }

        static function makeBd(w:int, h:int, transparent:Boolean):BitmapData {
            var bd:BitmapData = new BitmapData(w, h, transparent, transparent ? 0x00000000 : 0xFFFFFFFF);
            for (var y:int = 0; y < h; y++) {
                for (var x:int = 0; x < w; x++) {
                    var a:int = transparent ? (128 + ((x + y) * 5) % 128) : 255;
                    var r:int = (x * 12) & 0xFF;
                    var g:int = (y * 12) & 0xFF;
                    var b:int = ((x + y) * 6) & 0xFF;
                    bd.setPixel32(x, y, (a << 24) | (r << 16) | (g << 8) | b);
                }
            }
            return bd;
        }

        static function describePng(ba:ByteArray):void {
            trace("class: " + getClassName(ba));
            trace("length > 0: " + (ba.length > 0));
            ba.position = 0;
            ba.endian = Endian.BIG_ENDIAN;
            var sig:Array = [];
            for (var i:int = 0; i < 8; i++) {
                sig.push(hex2(ba.readUnsignedByte()));
            }
            trace("PNG sig: " + sig.join(" "));
            describePngDims(ba);
        }

        static function describePngDims(ba:ByteArray):void {
            ba.position = 8;
            ba.endian = Endian.BIG_ENDIAN;
            var ihdrLen:uint = ba.readUnsignedInt();
            var ihdrType:String = ba.readUTFBytes(4);
            var w:uint = ba.readUnsignedInt();
            var h:uint = ba.readUnsignedInt();
            var bitDepth:uint = ba.readUnsignedByte();
            var colorType:uint = ba.readUnsignedByte();
            trace("IHDR len=" + ihdrLen + " type=" + ihdrType + " w=" + w + " h=" + h
                + " bitDepth=" + bitDepth + " colorType=" + colorType);
        }

        static function describeJpeg(ba:ByteArray):void {
            trace("class: " + getClassName(ba));
            trace("length > 0: " + (ba.length > 0));
            ba.position = 0;
            ba.endian = Endian.BIG_ENDIAN;
            var b0:int = ba.readUnsignedByte();
            var b1:int = ba.readUnsignedByte();
            trace("SOI: " + hex2(b0) + " " + hex2(b1));
            while (ba.bytesAvailable >= 4) {
                var m1:int = ba.readUnsignedByte();
                if (m1 != 0xFF) continue;
                var marker:int = ba.readUnsignedByte();
                if (marker == 0 || marker == 0xFF) continue;
                if (marker == 0xD8) continue;
                if (marker == 0xD9) {
                    trace("EOI reached before SOF");
                    return;
                }
                var isSof:Boolean = (marker >= 0xC0 && marker <= 0xCF
                    && marker != 0xC4 && marker != 0xC8 && marker != 0xCC);
                if (isSof) {
                    var segLen:uint = ba.readUnsignedShort();
                    var precision:int = ba.readUnsignedByte();
                    var hh:uint = ba.readUnsignedShort();
                    var ww:uint = ba.readUnsignedShort();
                    var comps:int = ba.readUnsignedByte();
                    trace("SOF segLen=" + segLen
                        + " precision=" + precision
                        + " w=" + ww + " h=" + hh
                        + " components=" + comps);
                    trace("EOI present: " + findEoi(ba));
                    return;
                }
                var segLen2:uint = ba.readUnsignedShort();
                if (segLen2 < 2 || segLen2 - 2 > ba.bytesAvailable) {
                    trace("malformed segment");
                    return;
                }
                ba.position += segLen2 - 2;
            }
            trace("no SOF found");
        }

        static function findEoi(ba:ByteArray):Boolean {
            var prev:int = -1;
            while (ba.bytesAvailable > 0) {
                var b:int = ba.readUnsignedByte();
                if (prev == 0xFF && b == 0xD9) return true;
                prev = b;
            }
            return false;
        }

        static function hex2(v:int):String {
            var s:String = v.toString(16);
            return s.length < 2 ? "0" + s : s;
        }

        static function getClassName(o:Object):String {
            if (o == null) return "null";
            return getQualifiedClassName(o);
        }
    }
}
