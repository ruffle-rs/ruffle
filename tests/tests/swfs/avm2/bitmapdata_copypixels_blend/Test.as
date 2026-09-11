package {

import flash.display.*;
import flash.geom.*;

public class Test extends Sprite {
    private static const PIXEL:Rectangle = new Rectangle(0, 0, 1, 1);
    private static const ORIGIN:Point = new Point(0, 0);

    private var target = new BitmapData(1, 1, true, 0);
    private var source = new BitmapData(1, 1, true, 0);
    private var alphaSource = new BitmapData(1, 1, true, 0);
    private var target_source = new BitmapData(2, 1, true, 0);
    private var target_source_alpha = new BitmapData(3, 1, true, 0);

    private var mismatches:uint = 0;

    public function Test() {
        // Exhaustive sweeps, they only report when the result disagrees
        // with the formula below.
        testColorChannels();
        testAlphaChannel();
        testSelfCopy();

        // Curated tables of actual values, they cover what the sweeps
        // above cannot check on their own.
        traceBlendTable();
        traceAlphaSourceTable();
        traceAlphaMultiplierSweep();

        trace("--- mismatches");
        trace(mismatches);
    }

    // Blending anything over an opaque destination yields an opaque result,
    // and an opaque pixel survives the premultiply/unmultiply roundtrip
    // untouched. So with an opaque destination getPixel32 hands back the
    // blend result as it was computed, which makes it possible to sweep the
    // colors exhaustively without depending on the accuracy of the roundtrip.
    private function testColorChannels() {
        for (var sa:uint = 0; sa < 256; sa++) {
            // Every channel is blended on its own, so three of them can be
            // used to probe three different color values at a time.
            for (var c:uint = 0; c < 256; c += 3) {
                var c1:uint = c;
                var c2:uint = (c + 1) & 0xff;
                var c3:uint = (c + 2) & 0xff;

                checkColorChannels(argb(0xff, c1, c2, c3), argb(sa, 0x00, 0x7f, 0xff));
                checkColorChannels(argb(0xff, 0x00, 0x7f, 0xff), argb(sa, c1, c2, c3));
            }
        }
    }

    private function checkColorChannels(dest:uint, src:uint) {
        var srcAlpha:uint = alphaOf(src);
        var expected:uint = argb(
            0xff,
            blendChannel(redOf(dest), redOf(src), srcAlpha),
            blendChannel(greenOf(dest), greenOf(src), srcAlpha),
            blendChannel(blueOf(dest), blueOf(src), srcAlpha));

        var actual:uint = blend(dest, src);
        if (actual != expected) {
            reportMismatch(hex(dest) + " x " + hex(src), expected, actual);
        }
    }

    // `dest` is expected to be premultiplied already, which for an opaque
    // destination means it is just the color as it was written.
    private function blendChannel(dest:uint, src:uint, srcAlpha:uint):uint {
        return premultiply(src, srcAlpha) + ((dest * (256 - srcAlpha)) >> 8);
    }

    private function premultiply(color:uint, a:uint):uint {
        return int((color * a + 127) / 255);
    }

    // The alpha channel is stored as-is, it is never unmultiplied on the way
    // out, so every pair of alphas can be checked directly.
    private function testAlphaChannel() {
        for (var da:uint = 0; da < 256; da++) {
            for (var sa:uint = 0; sa < 256; sa++) {
                var dest:uint = argb(da, 0x33, 0x66, 0x99);
                var src:uint = argb(sa, 0xcc, 0x99, 0x33);

                var expected:uint = sa + ((da * (256 - sa)) >> 8);
                var actual:uint = alphaOf(blend(dest, src));
                if (actual != expected) {
                    reportMismatch(hex(dest) + " x " + hex(src), expected, actual);
                }
            }
        }
    }

    // Blending a bitmap onto itself has to produce the same result as
    // blending two separate bitmaps.
    private function testSelfCopy() {
        var values = [0x00, 0x01, 0x7f, 0x80, 0xff];
        var alphaValues = [0x80000000, 0xff000000];

        for each (var da in values) {
        for each (var dc in values) {
        for each (var sa in values) {
        for each (var sc in values) {
            var dest:uint = argb(da, dc, dc, dc);
            var src:uint = argb(sa, sc, sc, sc);

            target_source.setPixel32(0, 0, dest);
            target_source.setPixel32(1, 0, src);
            target_source.copyPixels(target_source, new Rectangle(1, 0, 1, 1), ORIGIN, null, null, true);

            var expected:uint = blend(dest, src);
            var actual:uint = target_source.getPixel32(0, 0);
            if (actual != expected) {
                reportMismatch("[self copy] " + hex(dest) + " x " + hex(src), expected, actual);
            }

            for each (var alphaColor in alphaValues) {
                target_source_alpha.setPixel32(0, 0, dest);
                target_source_alpha.setPixel32(1, 0, src);
                target_source_alpha.setPixel32(2, 0, alphaColor);
                target_source_alpha.copyPixels(target_source_alpha, new Rectangle(1, 0, 1, 1), ORIGIN, target_source_alpha, new Point(2, 0), true);

                expected = blendWithAlpha(dest, src, alphaColor);
                actual = target_source_alpha.getPixel32(0, 0);
                if (actual != expected) {
                    reportMismatch("[self copy] " + hex(dest) + " x " + hex(src) + " x " + hex(alphaColor), expected, actual);
                }
            }
        }
        }
        }
        }
    }

    // Covers the premultiply/unmultiply roundtrip around the blend, which the
    // sweeps above deliberately avoid. The destination packs three different
    // color values into its three channels.
    private function traceBlendTable() {
        var alphas = [0x00, 0x01, 0x20, 0x7f, 0x80, 0xbd, 0xfe, 0xff];
        var colors = [0x00, 0x01, 0x80, 0xff];

        trace("--- blend");
        for each (var da in alphas) {
        for each (var sa in alphas) {
        for each (var sc in colors) {
            var dest:uint = argb(da, 0x00, 0x80, 0xff);
            var src:uint = argb(sa, sc, sc, sc);
            trace(hex(dest) + " x " + hex(src) + " -> " + hex(blend(dest, src)));
        }
        }
        }
    }

    private function traceAlphaSourceTable() {
        var alphas = [0x00, 0x01, 0x20, 0x7f, 0x80, 0xbd, 0xfe, 0xff];
        var colors = [0x00, 0x01, 0x80, 0xff];

        trace("--- blend with an alpha source");
        for each (var da in alphas) {
        for each (var sa in alphas) {
        for each (var sc in colors) {
            var dest:uint = argb(da, 0x00, 0x80, 0xff);
            var src:uint = argb(sa, sc, sc, sc);
            var result:uint = blendWithAlpha(dest, src, 0xff000000);
            trace(hex(dest) + " x " + hex(src) + " -> " + hex(result));
        }
        }
        }
    }

    private function traceAlphaMultiplierSweep() {
        trace("--- alpha multiplier");
        for (var i:uint = 0; i < 256; i++) {
            var alphaColor:uint = argb(i, 0, 0, 0);
            traceWithAlpha(0xff332211, 0x8000ff00, alphaColor);
            traceWithAlpha(0xff332211, 0xff00ff00, alphaColor);
        }
    }

    private function traceWithAlpha(dest:uint, src:uint, alphaColor:uint) {
        var result:uint = blendWithAlpha(dest, src, alphaColor);
        trace(hex(dest) + " x " + hex(src) + " x " + hex(alphaColor) + " -> " + hex(result));
    }

    private function blend(dest:uint, src:uint):uint {
        target.setPixel32(0, 0, dest);
        source.setPixel32(0, 0, src);
        target.copyPixels(source, PIXEL, ORIGIN, null, null, true);
        return target.getPixel32(0, 0);
    }

    private function blendWithAlpha(dest:uint, src:uint, alphaColor:uint):uint {
        target.setPixel32(0, 0, dest);
        source.setPixel32(0, 0, src);
        alphaSource.setPixel32(0, 0, alphaColor);
        target.copyPixels(source, PIXEL, ORIGIN, alphaSource, ORIGIN, true);
        return target.getPixel32(0, 0);
    }

    private function reportMismatch(what:String, expected:uint, actual:uint) {
        mismatches++;
        // A systematic failure would otherwise bury the rest of the output.
        if (mismatches <= 20) {
            trace("mismatch: " + what + " -> " + hex(actual) + ", expected " + hex(expected));
        }
    }

    private function argb(a:uint, r:uint, g:uint, b:uint):uint {
        return a * 0x1000000 + r * 0x10000 + g * 0x100 + b;
    }

    private function alphaOf(color:uint):uint {
        return (color >>> 24) & 0xff;
    }

    private function redOf(color:uint):uint {
        return (color >>> 16) & 0xff;
    }

    private function greenOf(color:uint):uint {
        return (color >>> 8) & 0xff;
    }

    private function blueOf(color:uint):uint {
        return color & 0xff;
    }

    private function hex(color:uint):String {
        return color.toString(16);
    }
}

}
