// Calls BitmapData.generateFilterRect with every kind of filter, with source
// rectangles that are fractional, partly or fully outside the bitmap, and with
// invalid arguments.
package {
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.filters.BitmapFilter;
    import flash.filters.BevelFilter;
    import flash.filters.BlurFilter;
    import flash.filters.ColorMatrixFilter;
    import flash.filters.ConvolutionFilter;
    import flash.filters.DisplacementMapFilter;
    import flash.filters.DropShadowFilter;
    import flash.filters.GlowFilter;
    import flash.filters.GradientBevelFilter;
    import flash.filters.GradientGlowFilter;
    import flash.filters.ShaderFilter;
    import flash.geom.Point;
    import flash.geom.Rectangle;

    public class Test extends Sprite {
        private var bitmap:BitmapData = new BitmapData(100, 80, true, 0);

        public function Test() {
            var source:Rectangle = new Rectangle(10, 10, 25, 25);

            trace("// How far a blur grows the rectangle per quality, for sizes 0 to 6 in steps of 0.25");
            for (var quality:int = 0; quality <= 15; quality++) {
                var growths:Array = [];

                for (var size:Number = 0; size <= 6; size += 0.25) {
                    growths.push(source.x - bitmap.generateFilterRect(source, new BlurFilter(size, 0, quality)).x);
                }

                trace("quality " + quality + ": " + growths.join(" "));
            }

            trace("// How far a glow grows the rectangle per quality, for sizes 0 to 6 in steps of 0.25");
            for (quality = 0; quality <= 15; quality++) {
                growths = [];

                for (size = 0; size <= 6; size += 0.25) {
                    growths.push(source.x - bitmap.generateFilterRect(source, new GlowFilter(0, 1, size, 0, 1, quality)).x);
                }

                trace("quality " + quality + ": " + growths.join(" "));
            }

            trace("// Blur");
            show("blur 10x2 quality 2", source, new BlurFilter(10, 2, 2));
            show("blur 255 quality 15", source, new BlurFilter(255, 255, 15));

            trace("// Glow");
            show("glow 6 quality 2", source, new GlowFilter(0xFF0000, 1, 6, 6, 2, 2));
            show("glow 6 quality 2, inner", source, new GlowFilter(0xFF0000, 1, 6, 6, 2, 2, true));
            show("glow 6 quality 2, knockout", source, new GlowFilter(0xFF0000, 1, 6, 6, 2, 2, false, true));

            trace("// Filters with an offset copy");
            for each (var distance:Number in [0, 3, 10, -5]) {
                for each (var angle:Number in [0, 30, 45, 90, 135, 180, 270]) {
                    var label:String = "distance " + distance + ", angle " + angle;
                    show("drop shadow, " + label, source, new DropShadowFilter(distance, angle, 0, 1, 4, 4, 1, 1));
                    show("gradient glow, " + label, source, new GradientGlowFilter(distance, angle, [0, 0xFF0000], [0, 1], [0, 255], 4, 4, 1, 1));
                    show("bevel, " + label, source, new BevelFilter(distance, angle, 0xFFFFFF, 1, 0, 1, 4, 4, 1, 1));
                    show("gradient bevel, " + label, source, new GradientBevelFilter(distance, angle, [0, 0xFF0000], [0, 1], [0, 255], 4, 4, 1, 1));
                }
            }
            show("drop shadow, blur 2x6 quality 2", source, new DropShadowFilter(4, 45, 0, 1, 2, 6, 1, 2));
            show("drop shadow, inner, knockout", source, new DropShadowFilter(4, 45, 0, 1, 4, 4, 1, 1, true, true));
            show("bevel, outer", source, new BevelFilter(4, 45, 0xFFFFFF, 1, 0, 1, 4, 4, 1, 1, "outer"));

            trace("// Convolution");
            show("convolution 0x0", source, new ConvolutionFilter());
            show("convolution 1x1", source, new ConvolutionFilter(1, 1, [1]));
            show("convolution 2x2", source, new ConvolutionFilter(2, 2, [1, 1, 1, 1]));
            show("convolution 3x3", source, new ConvolutionFilter(3, 3, []));
            show("convolution 5x1", source, new ConvolutionFilter(5, 1, []));
            show("convolution 4x6", source, new ConvolutionFilter(4, 6, []));

            trace("// Inner and outer filters near the corner");
            var corner:Rectangle = new Rectangle(90, 70, 10, 10);

            for each (var inner:Boolean in [false, true]) {
                show("glow, inner " + inner, corner, new GlowFilter(0, 1, 20, 20, 1, 1, inner));
                show("drop shadow, inner " + inner, corner, new DropShadowFilter(30, 45, 0, 1, 20, 20, 1, 1, inner));
            }

            for each (var type:String in ["inner", "outer", "full"]) {
                show("bevel, " + type, corner, new BevelFilter(30, 45, 0xFFFFFF, 1, 0, 1, 20, 20, 1, 1, type));
                show("gradient glow, " + type, corner, new GradientGlowFilter(30, 45, [0, 0xFF0000], [0, 1], [0, 255], 20, 20, 1, 1, type));
                show("gradient bevel, " + type, corner, new GradientBevelFilter(30, 45, [0, 0xFF0000], [0, 1], [0, 255], 20, 20, 1, 1, type));
            }

            trace("// Offset copies at large negative distances to the left");
            for each (var far:Number in [-255.5, -256, -256.004, -261.796, -1000.5]) {
                show("drop shadow, distance " + far, source, new DropShadowFilter(far, 0, 0, 1, 0, 0, 1, 1));
                show("bevel, distance " + far, source, new BevelFilter(far, 0, 0xFFFFFF, 1, 0, 1, 0, 0, 1, 1, "outer"));
            }

            trace("// Displacement maps");
            var middle:Rectangle = new Rectangle(40, 30, 20, 20);
            var origin:Point = new Point(0, 0);
            var red:BitmapData = new BitmapData(4, 4, false, 0xFF0000);
            var gray:BitmapData = new BitmapData(4, 4, false, 0x808080);
            var redAndGreen:BitmapData = new BitmapData(2, 1, false, 0);
            redAndGreen.setPixel(0, 0, 0xFF8000);
            redAndGreen.setPixel(1, 0, 0x80FF00);
            var transparent:BitmapData = new BitmapData(2, 2, true, 0);
            transparent.setPixel32(0, 0, 0x01FFFFFF);
            var disposed:BitmapData = new BitmapData(4, 4, false, 0xFF0000);
            disposed.dispose();

            for each (var mode:String in ["wrap", "clamp", "ignore", "color"]) {
                show("displacement map, mode " + mode, middle, new DisplacementMapFilter(red, origin, 1, 1, 100, 50, mode));
            }

            show("displacement map, gray", middle, new DisplacementMapFilter(gray, origin, 1, 1, 50, 50, "color"));
            show("displacement map, red x, green y", middle, new DisplacementMapFilter(redAndGreen, origin, 1, 2, 100, 30, "color"));
            show("displacement map, transparent", middle, new DisplacementMapFilter(transparent, origin, 1, 8, 100, 100, "color"));

            for each (var scale:Number in [-100, 2.01, 2.02, 65535, 1e9, Infinity, NaN]) {
                show("displacement map, scale " + scale, middle, new DisplacementMapFilter(red, origin, 1, 1, scale, scale, "color"));
            }

            for each (var component:int in [0, 3, 16, 257, -254]) {
                show("displacement map, component " + component, middle, new DisplacementMapFilter(red, origin, component, component, 100, 100, "color"));
            }

            show("displacement map, map point (80, 60)", middle, new DisplacementMapFilter(red, new Point(80, 60), 1, 1, 100, 50, "color"));
            show("displacement map, map point (-2, -2)", middle, new DisplacementMapFilter(red, new Point(-2, -2), 1, 1, 100, 50, "color"));
            show("displacement map, map point (45, 35)", middle, new DisplacementMapFilter(red, new Point(45, 35), 1, 1, 100, 50, "color"));
            show("displacement map, no map", middle, new DisplacementMapFilter(null, null, 1, 2, 10, 10));
            show("displacement map, disposed map", middle, new DisplacementMapFilter(disposed, origin, 1, 1, 100, 100, "color"));

            trace("// Shader filters");
            var shaderFilter:ShaderFilter = new ShaderFilter();
            show("shader without extensions", source, shaderFilter);
            shaderFilter.leftExtension = 5;
            shaderFilter.topExtension = 6;
            shaderFilter.rightExtension = 7;
            shaderFilter.bottomExtension = 8;
            show("shader with extensions", new Rectangle(200, 200, 10, 10), shaderFilter);

            trace("// Color matrix");
            show("color matrix", source, new ColorMatrixFilter());

            trace("// Fractional source rectangles");
            var blur:BlurFilter = new BlurFilter(4, 4, 1);
            show("10.4, 10.6, 25.3, 25.2", new Rectangle(10.4, 10.6, 25.3, 25.2), blur);
            show("10.49, 10.51, 20.02, 20", new Rectangle(10.49, 10.51, 20.02, 20), blur);

            for each (var edge:Number in [0.5, 1.5, 10.5, 11.5, 12.5, 13.5]) {
                show("x and width " + edge, new Rectangle(edge, 20, edge, 10), blur);
                show("width " + edge, new Rectangle(20, 20, edge, 10), blur);
            }

            trace("// Source rectangles partly or fully outside the bitmap");
            show("whole bitmap", bitmap.rect, blur);
            show("covering the bitmap", new Rectangle(-50, -50, 300, 300), blur);
            show("over the right and bottom edges", new Rectangle(90, 70, 20, 20), blur);
            show("over the left and top edges", new Rectangle(-10, -5, 20, 20), blur);
            show("starting at the right edge", new Rectangle(100, 10, 10, 10), blur);
            show("empty", new Rectangle(10, 10, 0, 0), blur);
            show("negative size", new Rectangle(10, 10, -5, -5), blur);
            show("outside", new Rectangle(200, 200, 10, 10), blur);
            show("outside, color matrix", new Rectangle(200, 200, 10, 10), new ColorMatrixFilter());

            trace("// Source rectangles with non-finite or huge values");
            var huge:Array = [
                new Rectangle(NaN, 0, 10, 10),
                new Rectangle(0, 0, Infinity, 10),
                new Rectangle(0, 0, 2147483648, 2147483648),
                new Rectangle(-2147483648, -2147483648, 4294967296, 4294967296),
                new Rectangle(2147483640, 0, 100, 10),
                new Rectangle(-2147483649, 0, 2147483700, 10)
            ];

            for each (var hugeRect:Rectangle in huge) {
                show(String(hugeRect), hugeRect, blur);
            }

            trace("// Invalid arguments");
            tryCall("null source rectangle", function():* { return bitmap.generateFilterRect(null, blur); });
            tryCall("null filter", function():* { return bitmap.generateFilterRect(source, null); });
            bitmap.dispose();
            tryCall("disposed bitmap", function():* { return bitmap.generateFilterRect(source, blur); });
            tryCall("disposed bitmap, null source rectangle", function():* { return bitmap.generateFilterRect(null, blur); });
        }

        private function show(label:String, source:Rectangle, filter:BitmapFilter):void {
            trace(label + ": " + bitmap.generateFilterRect(source, filter));
        }

        private function tryCall(label:String, call:Function):void {
            try {
                trace(label + ": " + call());
            } catch (e:Error) {
                trace(label + ": " + e);
            }
        }
    }
}
