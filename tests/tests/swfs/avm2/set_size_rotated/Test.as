// Tests setting width and height of shapes that are scaled, rotated or both.
//
// For every combination of size, scaleX and rotation below, one new shape
// gets a new width and another one a new height. The edge cases at the end
// cover negative scaleY, negative sizes and other combinations. Each line
// traces width, height, scaleX and scaleY before and after the assignment.
// A rotated video and a text field with negative and zero sizes follow.
package {
    import flash.display.DisplayObject;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.media.Video;
    import flash.text.TextField;

    public class Test extends Sprite {
        public function Test() {
            var sizes:Array = [0, 50, 100];
            var scales:Array = [-100, -50, 0, 50, 200];
            var rotations:Array = [0, 10, 45, 90];

            for each (var w:Number in sizes) {
                for each (var h:Number in sizes) {
                    for each (var xscale:Number in scales) {
                        for each (var rotation:Number in rotations) {
                            test(w, h, xscale, 100, rotation, "width", 150);
                            test(w, h, xscale, 100, rotation, "height", 80);
                        }
                    }
                }
            }

            var edgeCases:Array = [
                // [  w,  h, xscale, yscale, rotation, property, value]
                [100, 50,    100,    -50,        0,  "width",   150],
                [100, 50,    100,    -50,       30, "height",    80],
                [100, 50,    -50,   -100,       45,  "width",   150],
                [100, 50,    200,     50,       30, "height",   100],
                [100, 50,    100,    100,      170,  "width",   150],
                [100, 50,    100,    100,     -135, "height",    80],
                [100, 50,    100,    100,       30,  "width",     0],
                [100, 50,    100,    100,       30, "height",   -10],
                [100, 50,    100,    100,        0,  "width",   -10]
            ];

            for each (var c:Array in edgeCases) {
                var edgeW:Number = c[0];
                var edgeH:Number = c[1];
                var edgeXscale:Number = c[2];
                var edgeYscale:Number = c[3];
                var edgeRotation:Number = c[4];
                var property:String = c[5];
                var value:Number = c[6];

                test(edgeW, edgeH, edgeXscale, edgeYscale, edgeRotation, property, value);
            }

            var video:Video = new Video(100, 100);
            video.rotation = 45;
            var videoBefore:String = describe(video);
            video.width = 200;
            trace("video 100x100 rot 45 width = 200: " + videoBefore + " -> " + describe(video));

            var field:TextField = new TextField();
            field.width = 100;
            field.height = 50;

            for each (var size:Number in [-10, 0]) {
                field.width = size;
                trace("text field width = " + size + ": width " + field.width + ", height " + field.height);

                field.height = size;
                trace("text field height = " + size + ": width " + field.width + ", height " + field.height);
            }
        }

        private function test(w:Number, h:Number, xscale:Number, yscale:Number, rotation:Number, property:String, value:Number):void {
            var shape:Shape = new Shape();
            shape.graphics.beginFill(0xFF0000);
            shape.graphics.moveTo(0, 0);
            shape.graphics.lineTo(w, 0);
            shape.graphics.lineTo(w, h);
            shape.graphics.lineTo(0, h);
            shape.graphics.endFill();
            addChild(shape);

            shape.scaleX = xscale / 100;
            shape.scaleY = yscale / 100;
            shape.rotation = rotation;

            var label:String = w + "x" + h + " scale " + xscale + "/" + yscale + " rot " + rotation;
            var before:String = describe(shape);
            shape[property] = value;
            trace(label + " " + property + " = " + value + ": " + before + " -> " + describe(shape));
        }

        private function describe(object:DisplayObject):String {
            return "width " + object.width + ", height " + object.height + ", scaleX " + object.scaleX + ", scaleY " + object.scaleY;
        }
    }
}
