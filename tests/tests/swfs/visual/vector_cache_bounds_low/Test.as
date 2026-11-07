package {
    import flash.display.MovieClip;
    import flash.display.Sprite;
    import flash.display.StageAlign;
    import flash.display.StageQuality;
    import flash.display.StageScaleMode;
    import flash.events.Event;
    import flash.filters.ColorMatrixFilter;
    import flash.geom.Rectangle;

    [SWF(width="550", height="240", frameRate="30", backgroundColor="#EEEEEE")]
    public class Test extends MovieClip {
        private var markers:Array = [];
        private var bodies:Array = [];
        private var frame:int = 0;

        public function Test() {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;
            stage.quality = StageQuality.LOW;

            for (var i:int = 0; i < 3; i++) {
                var container:Sprite = new Sprite();
                container.x = 35 + i * 160;
                container.y = 105;
                addChild(container);

                // All visible artwork is vector fill geometry. There are no
                // strokes, bitmaps, fonts, masks, or scroll rectangles.
                var body:Sprite = new Sprite();
                body.graphics.beginFill(0xFFFFFF);
                body.graphics.drawRect(0, 0, 64, 64);
                body.graphics.endFill();
                body.graphics.beginFill(0x000000);
                body.graphics.drawRect(0, 0, 64, 4);
                body.graphics.drawRect(0, 60, 64, 4);
                body.graphics.drawRect(0, 4, 4, 56);
                body.graphics.drawRect(60, 4, 4, 56);
                body.graphics.drawRect(12, 12, 4, 40);
                body.graphics.drawRect(48, 12, 4, 40);
                body.graphics.endFill();
                body.graphics.beginFill(0x00FFFF);
                body.graphics.drawRect(20, 20, 24, 24);
                body.graphics.endFill();
                // Diagonal fill probe
                body.graphics.beginFill(0xFF0000);
                body.graphics.moveTo(8.25, 8);
                body.graphics.lineTo(56.25, 8);
                body.graphics.lineTo(8.25, 56);
                body.graphics.lineTo(8.25, 8);
                body.graphics.endFill();
                container.addChild(body);
                bodies.push(body);

                // This visible child remains outside the stage. Its movement
                // changes the container's bounds without moving the body.
                var marker:Sprite = new Sprite();
                marker.graphics.beginFill(0x000000);
                marker.graphics.drawRect(0, 0, 1, 1);
                marker.graphics.endFill();
                marker.x = -600;
                marker.y = -600;
                container.addChild(marker);
                markers.push(marker);

                if (i == 1) {
                    container.cacheAsBitmap = true;
                } else if (i == 2) {
                    // Swap red and blue, turning cyan into yellow. This
                    // forces a non-identity filter without expanding bounds.
                    container.filters = [new ColorMatrixFilter([
                        0, 0, 1, 0, 0,
                        0, 1, 0, 0, 0,
                        1, 0, 0, 0, 0,
                        0, 0, 0, 1, 0
                    ])];
                }
            }

            addEventListener(Event.ENTER_FRAME, advance);
        }

        private function advance(event:Event):void {
            frame++;
            // Exact quarter-pixel positions avoid floating-point ambiguity
            // when Flash and Ruffle convert coordinates to twips.
            for each (var marker:Sprite in markers) {
                marker.x = -600 + (frame % 4) * 0.25;
                marker.y = -600 + ((frame * 3) % 4) * 0.25;
            }

            if (frame == 1 || frame == 5 || frame == 6 || frame == 7
                    || frame == 8 || frame == 90) {
                trace("frame=" + frame);
                for (var i:int = 0; i < bodies.length; i++) {
                    var bounds:Rectangle = bodies[i].getBounds(stage);
                    trace("case=" + i + " body="
                        + bounds.x.toFixed(2) + "," + bounds.y.toFixed(2) + ","
                        + bounds.width.toFixed(2) + "," + bounds.height.toFixed(2));
                }
            }

            if (frame == 90) {
                removeEventListener(Event.ENTER_FRAME, advance);
                trace("DONE");
                stop();
            }
        }
    }
}
