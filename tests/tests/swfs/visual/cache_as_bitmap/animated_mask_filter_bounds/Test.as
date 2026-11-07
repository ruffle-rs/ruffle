package {
    import flash.display.Bitmap;
    import flash.display.BlendMode;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.display.StageAlign;
    import flash.display.StageScaleMode;
    import flash.events.Event;
    import flash.filters.ColorMatrixFilter;
    import flash.geom.Rectangle;
    import flash.system.fscommand;

    [SWF(width="480", height="300", frameRate="30", backgroundColor="#A0A0A0")]
    public class Test extends MovieClip {
        [Embed(source="maskee.png")]
        private var Maskee:Class;

        private var containers:Array = [];
        private var bitmaps:Array = [];
        private var masks:Array = [];
        private var frame:int = 0;

        public function Test() {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;

            trace("Animated mask / filter bounds");
            trace("Top row, left to right: ColorMatrix+HardLight, ColorMatrix, HardLight");
            trace("Bottom row: no effects, ColorMatrix+moving clipping edge, ColorMatrix+moving bitmap");

            for (var i:int = 0; i < 6; i++) {
                var container:Sprite = new Sprite();
                container.x = 40 + (i % 3) * 160;
                container.y = 35 + int(i / 3) * 150;
                addChild(container);

                var bitmap:Bitmap = new Maskee();
                bitmap.scaleX = bitmap.scaleY = 4;
                container.addChild(bitmap);

                var maskShape:Shape = new Shape();
                container.addChild(maskShape);
                bitmap.mask = maskShape;
                drawMask(maskShape, -12, -12, 104, 104);

                if (i == 0 || i == 1 || i == 4 || i == 5) {
                    container.filters = [new ColorMatrixFilter([
                        0.8, 0, 0, 0, 12,
                        0, 0.9, 0, 0, 6,
                        0, 0, 0.7, 0, 18,
                        0, 0, 0, 1, 0
                    ])];
                }

                if (i == 0 || i == 2) {
                    container.blendMode = BlendMode.HARDLIGHT;
                }


                containers.push(container);
                bitmaps.push(bitmap);
                masks.push(maskShape);
            }

            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function drawMask(
            maskShape:Shape,
            x:Number,
            y:Number,
            width:Number,
            height:Number
        ):void {
            maskShape.graphics.clear();
            maskShape.graphics.beginFill(0x000000);
            maskShape.graphics.drawRect(x, y, width, height);
            maskShape.graphics.endFill();
        }

        private function bounds(rect:Rectangle):String {
            return rect.x.toFixed(2) + "," +
                rect.y.toFixed(2) + "," +
                rect.width.toFixed(2) + "," +
                rect.height.toFixed(2);
        }

        private function onFrame(event:Event):void {
            frame++;

            var phase:int = (frame - 1) % 40;
            var controlPhase:int = int((frame - 1) / 2) % 4;
            var delta:Number = phase * 0.05;

            for (var i:int = 0; i < 6; i++) {
                var maskShape:Shape = masks[i];
                var bitmap:Bitmap = bitmaps[i];

                if (i == 4) {
                    // Intentional visible clipping motion.
                    drawMask(maskShape, controlPhase * 10, 0, 40, 80);
                } else {
                    // Changing mask bounds; bitmap remains fully covered.
                    drawMask(
                        maskShape,
                        -12 - delta,
                        -12 - delta,
                        104 + delta * 2,
                        104 + delta * 2
                    );
                }

                if (i == 5) {
                    // Intentional bitmap motion.
                    bitmap.x = controlPhase * 2;
                }
            }

            if (
                frame == 1 || frame == 11 || frame == 19 ||
                frame == 35 || frame == 40 || frame == 60 ||
                frame == 123
            ) {
                trace("frame=" + frame + " delta=" + delta.toFixed(2));

                for (i = 0; i < 6; i++) {
                    trace(
                        "case=" + i +
                        " bitmap=" +
                        bounds(Bitmap(bitmaps[i]).getBounds(this)) +
                        " mask=" +
                        bounds(Shape(masks[i]).getBounds(this)) +
                        " container=" +
                        bounds(Sprite(containers[i]).getBounds(this))
                    );
                }
            }

            if (frame == 11 || frame == 19 || frame == 35) {
                fscommand("captureImage", "frame" + frame);
            }
            if (frame == 123) {
                trace("DONE");
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
