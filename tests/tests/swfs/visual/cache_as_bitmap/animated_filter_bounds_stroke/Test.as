package {
    import flash.display.*;
    import flash.events.Event;
    import flash.filters.*;

    [SWF(width="640", height="240", frameRate="30", backgroundColor="#FFFFFF")]
    public class Test extends MovieClip {

        private var markers:Array = [];
        private var frame:int = 0;
        private var group:Sprite = new Sprite();

        public function Test() {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;

            stage.quality = StageQuality.HIGH;
            addChild(group);
            var clip:Shape = new Shape();
            clip.graphics.beginFill(0);

            for (var i:int = 0; i < 4; i++) {
                var p:Sprite = new Sprite();
                p.x = 30 + i * 160;
                p.y = 100;
                p.scaleX = p.scaleY = 0.8;
                group.addChild(p);
                clip.graphics.drawRect(p.x + 4, p.y + 4, 56, 56);

                var pixels:Array = [
                    0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444,
                    0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888,
                    0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000,
                    0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
                    0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00,
                    0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff,
                    0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff,
                    0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff,
                    0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00,
                    0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000,
                    0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444,
                    0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888,
                    0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000,
                    0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
                    0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00,
                    0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffffff00, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff, 0xffff00ff,
                    0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff, 0xff00ffff,
                    0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff000000, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff, 0xff0000ff,
                    0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff888888, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00, 0xff00ff00,
                    0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xff444444, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000, 0xffff0000];
                var data:BitmapData = new BitmapData(20, 20, true, 0);
                for (var pixel:int = 0; pixel < pixels.length; pixel++) {
                    data.setPixel32(pixel % 20, int(pixel / 20), pixels[pixel]);
                }
                var body:Bitmap = new Bitmap(data);
                body.scaleX = body.scaleY = 4;
                p.addChild(body);

                var marker:Shape = new Shape();
                marker.graphics.lineStyle(1, 0xFF8800);
                marker.graphics.beginFill(0xFF8800);
                marker.graphics.drawRect(-25, -6, 50, 12);
                marker.graphics.endFill();

                if (i == 2) {
                    // Control: marker is outside the filtered container.
                    marker.x = p.x + 40 * 0.8;
                    marker.y = p.y - 25 * 0.8;
                    marker.scaleX = marker.scaleY = 0.8;
                    group.addChild(marker);
                } else {
                    marker.x = 40;
                    marker.y = -25;
                    p.addChild(marker);
                }

                if (i == 1 || i == 2) {
                    p.filters = [
                        new GlowFilter(0, 1, 1.5, 1.5, 15, 2, false, false)
                    ];
                } else if (i == 3) {
                    p.filters = [new ColorMatrixFilter([
                        0.8, 0, 0, 0, 12,
                        0, 0.9, 0, 0, 6,
                        0, 0, 0.7, 0, 18,
                        0, 0, 0, 1, 0
                    ])];
                }

                markers.push(marker);
            }

            clip.graphics.endFill();
            addChild(clip);
            group.mask = clip;
            addEventListener(Event.ENTER_FRAME, step);
        }

        private function step(event:Event):void {
            frame++;

            for (var i:int = 0; i < 4; i++) {
                Shape(markers[i]).rotation = (frame - 1) * 7;
            }

            if (frame == 90) {
                removeEventListener(Event.ENTER_FRAME, step);
            }
        }
    }
}
