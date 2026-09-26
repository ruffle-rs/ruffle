package {
    import flash.display.Sprite;
    import flash.filters.GradientBevelFilter;
    import flash.filters.GradientGlowFilter;

    public class Test extends Sprite {
        function testGradients(filterClass:Class, label:String, colors:*, alphas:*, ratios:*):void {
            trace("==" + label);
            try {
                filters = [new filterClass(4, 45, colors, alphas, ratios)];
                var filter:* = filters[0];
                trace("colors: " + filter.colors);
                trace("alphas: " + filter.alphas);
                var alphaBytes:Array = [];
                var alphaInv:Array = [];
                if (filter.alphas != null) {
                    for each (var alpha:* in filter.alphas) {
                        var n:Number = Number(alpha);
                        alphaBytes.push(Math.round(n * 255));
                        alphaInv.push(1 / n);
                    }
                }
                trace("alpha bytes: " + alphaBytes);
                trace("alpha inv: " + alphaInv);
                trace("ratios: " + filter.ratios);
            } catch (e:Error) {
                trace("error: " + e.getStackTrace());
            }
        }

        function testFull(filterClass:Class, label:String, distance:*, angle:*, colors:*, alphas:*, ratios:*, blurX:*, blurY:*, strength:*, quality:*, type:*, knockout:*):void {
            trace("==" + label);
            try {
                filters = [new filterClass(distance, angle, colors, alphas, ratios, blurX, blurY, strength, quality, type, knockout)];
                var filter:* = filters[0];
                trace("distance: " + filter.distance);
                trace("angle: " + filter.angle);
                trace("colors: " + filter.colors);
                trace("alphas: " + filter.alphas);
                var alphaBytes:Array = [];
                if (filter.alphas != null) {
                    for each (var alpha:* in filter.alphas) {
                        alphaBytes.push(Math.round(Number(alpha) * 255));
                    }
                }
                trace("alpha bytes: " + alphaBytes);
                trace("ratios: " + filter.ratios);
                trace("blurX: " + filter.blurX + ", blurY: " + filter.blurY);
                trace("strength: " + filter.strength + ", quality: " + filter.quality + ", type: " + filter.type + ", knockout: " + filter.knockout);
            } catch (e:Error) {
                trace("error: " + e.getStackTrace());
            }
        }

        public function Test() {
            for each (var filterClass:Class in [GradientBevelFilter, GradientGlowFilter]) {
                trace(filterClass);

                var baseColors:Array = [0x123456, 0xABCDEF, 0x654321];
                var baseAlphas:Array = [0, 0.2, 1];
                var baseRatios:Array = [17, 128, 239];

                testGradients(filterClass, "baseline", baseColors, baseAlphas, baseRatios);

                // Negative ratios clamp to 0.
                testGradients(filterClass, "ratios negative", baseColors, baseAlphas, [-1, -128, -1e10]);
                testGradients(filterClass, "ratios over 255", baseColors, baseAlphas, [255, 256, 300]);

                // Lengths: truncate to min(colors, ratios); missing alphas pad with 1.
                testGradients(filterClass, "colors too few", [0x123456], baseAlphas, baseRatios);
                testGradients(filterClass, "alphas too few", baseColors, [0.5], baseRatios);
                testGradients(filterClass, "ratios too few", baseColors, baseAlphas, [128]);
                testGradients(filterClass, "colors too many", [0x111111, 0x222222, 0x333333, 0x444444], [0, 0.5, 1], [0, 128, 255]);
                testGradients(filterClass, "null arrays", null, null, null);

                // Alpha saturation at the 0..1 boundaries.
                testGradients(filterClass, "alphas out of range", baseColors, [-0.001, 1.001, 1.5], baseRatios);

                // --- other constructor args ---
                testFull(filterClass, "defaults", 4.0, 45, null, null, null, 4.0, 4.0, 1, 1, "inner", false);
                // Quality clamps to 0..15.
                testFull(filterClass, "quality negative", 4, 45, baseColors, baseAlphas, baseRatios, 4, 4, 1, -1, "inner", false);
                testFull(filterClass, "quality overflow", 4, 45, baseColors, baseAlphas, baseRatios, 4, 4, 1, 16, "inner", false);
                // Null type throws #2007 in Flash.
                testFull(filterClass, "type outer", 4, 45, baseColors, baseAlphas, baseRatios, 4, 4, 1, 1, "outer", false);
                testFull(filterClass, "type null", 4, 45, baseColors, baseAlphas, baseRatios, 4, 4, 1, 1, null, false);
                testFull(filterClass, "strength huge", 4, 45, baseColors, baseAlphas, baseRatios, 4, 4, 1e10, 1, "inner", false);
            }
        }
    }
}
