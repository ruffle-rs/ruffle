package {
    import flash.display.*;
    import flash.geom.Matrix;
    import flash.geom.Point;
    import flash.geom.Rectangle;
    import flash.text.TextField;

    [SWF(width="450", height="360", backgroundColor="#808080")]
    public class Test extends Sprite {
        private static const W:int = 20;

        public function Test() {
            // [scrollX, scrollY, matrix]
            var cases:Array = [
                [0, 0, new Matrix()],
                [400, 0, new Matrix()],
                [0, 400, new Matrix()],
                [400, 400, new Matrix()],
                [-500, -500, new Matrix()],
                [300, 300, new Matrix(2, 0, 0, 2)],
                [400, 0, new Matrix(0, 1, -1, 0)],
                [300, 300, new Matrix(-1, 0, 0, -1)]
            ];
            var kinds:Array = ["bitmap", "shape", "textfield", "sprite", "parent", "nested"];

            for (var r:int = 0; r < kinds.length; r++) {
                for (var c:int = 0; c < cases.length; c++) {
                    var cs:Array = cases[c];
                    make(kinds[r], cs[0], cs[1], cs[2], 30 + c * 50, 30 + r * 50);
                }
            }

            // Objects partially outside of the stage.
            make("shape", 400, 0, new Matrix(), -10, 100);
            make("bitmap", 0, 400, new Matrix(), 200, -10);
            make("bitmap", 400, 400, new Matrix(), 440, 200);
            make("shape", -500, -500, new Matrix(), 200, 350);
            make("textfield", 400, 400, new Matrix(), -10, -10);
            make("bitmap", 300, 300, new Matrix(2, 0, 0, 2), 430, 100);
        }

        private function make(kind:String, sx:int, sy:int, m:Matrix, tx:Number, ty:Number):void {
            var negative:Boolean = sx < 0 || sy < 0;
            var obj:DisplayObject;
            if (kind == "bitmap") {
                if (negative) return;
                obj = makeBitmap(sx, sy);
            } else if (kind == "shape") {
                var shape:Shape = new Shape();
                quads(shape.graphics, sx, sy);
                obj = shape;
            } else if (kind == "textfield") {
                // Device font text fields aren't rendered when rotated or flipped.
                if (negative || m.a <= 0 || m.b != 0) return;
                var tf:TextField = new TextField();
                tf.selectable = false;
                tf.background = true;
                tf.backgroundColor = 0xFF00FF;
                tf.width = sx + 15;
                tf.height = sy + 15;
                obj = tf;
            } else if (kind == "sprite") {
                var sprite:Sprite = new Sprite();
                quads(sprite.graphics, sx, sy);
                obj = sprite;
            } else if (kind == "parent") {
                var parentSprite:Sprite = new Sprite();
                var child:Shape = new Shape();
                quads(child.graphics, sx, sy);
                parentSprite.addChild(child);
                obj = parentSprite;
            } else if (kind == "nested") {
                if (negative) return;
                var inner:Bitmap = makeBitmap(sx, sy);
                inner.scrollRect = new Rectangle(sx, sy, W, W);
                var size:Point = place(inner, m, 500, 500);
                var outer:Sprite = new Sprite();
                outer.addChild(inner);
                outer.scrollRect = new Rectangle(500, 500, size.x, size.y);
                outer.x = tx;
                outer.y = ty;
                addChild(outer);
                return;
            }
            obj.scrollRect = new Rectangle(sx, sy, W, W);
            place(obj, m, tx, ty);
            addChild(obj);
        }

        // Places the object so that the top left corner of its transformed
        // scroll rect is at (tx, ty), returns the size of the transformed scroll rect.
        private function place(obj:DisplayObject, m:Matrix, tx:Number, ty:Number):Point {
            var m2:Matrix = m.clone();
            var corners:Array = [new Point(0, 0), new Point(W, 0), new Point(0, W), new Point(W, W)];
            var minX:Number = Infinity, minY:Number = Infinity;
            var maxX:Number = -Infinity, maxY:Number = -Infinity;
            for each (var p:Point in corners) {
                var q:Point = m.deltaTransformPoint(p);
                minX = Math.min(minX, q.x);
                minY = Math.min(minY, q.y);
                maxX = Math.max(maxX, q.x);
                maxY = Math.max(maxY, q.y);
            }
            m2.tx = tx - minX;
            m2.ty = ty - minY;
            obj.transform.matrix = m2;
            return new Point(maxX - minX, maxY - minY);
        }

        private function makeBitmap(sx:int, sy:int):Bitmap {
            var bd:BitmapData = new BitmapData(sx + W, sy + W, false, 0);
            var colors:Array = [0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00];
            for (var i:int = 0; i < 4; i++) {
                bd.fillRect(new Rectangle(sx + (i % 2) * W / 2, sy + int(i / 2) * W / 2, W / 2, W / 2), colors[i]);
            }
            return new Bitmap(bd);
        }

        private function quads(g:Graphics, sx:int, sy:int):void {
            var colors:Array = [0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00];
            for (var i:int = 0; i < 4; i++) {
                g.beginFill(colors[i]);
                g.drawRect(sx + (i % 2) * W / 2, sy + int(i / 2) * W / 2, W / 2, W / 2);
                g.endFill();
            }
        }
    }
}
