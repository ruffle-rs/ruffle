package {
    import flash.display.*;
    import flash.events.Event;
    import flash.geom.Rectangle;

    public class Test extends MovieClip {
        private var objs:Array = [];
        private var other:Shape;
        private var frame:int = 0;

        public function Test() {
            var bmp:Bitmap = new Bitmap(new BitmapData(400, 400, false, 0xFF0000));
            setup("bitmap", bmp);

            var shape:Shape = new Shape();
            fill(shape.graphics);
            setup("shape", shape);

            var sprite:Sprite = new Sprite();
            fill(sprite.graphics);
            setup("sprite", sprite);

            var container:Sprite = new Sprite();
            var child:Shape = new Shape();
            fill(child.graphics);
            container.addChild(child);
            setup("sprite with child", container);

            other = new Shape();
            other.graphics.beginFill(0);
            other.graphics.drawRect(0, 0, 10, 10);
            other.graphics.endFill();
            addChild(other);

            trace("// before render");
            testAll();

            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function fill(g:Graphics):void {
            g.beginFill(0x00FF00);
            g.drawRect(0, 0, 400, 400);
            g.endFill();
        }

        private function setup(name:String, obj:DisplayObject):void {
            obj.name = name;
            obj.x = 100;
            obj.y = 50;
            obj.scrollRect = new Rectangle(150, 120, 30, 20);
            addChild(obj);
            objs.push(obj);
        }

        private function testPoint(obj:DisplayObject, x:Number, y:Number):void {
            trace("hitTestPoint(" + x + ", " + y + "): " + obj.hitTestPoint(x, y) + " " + obj.hitTestPoint(x, y, true));
        }

        private function testObject(obj:DisplayObject, x:Number, y:Number):void {
            other.x = x;
            other.y = y;
            trace("hitTestObject(" + x + ", " + y + "): " + obj.hitTestObject(other) + " " + other.hitTestObject(obj));
        }

        private function testAll():void {
            for each (var obj:DisplayObject in objs) {
                trace("// " + obj.name);
                testPoint(obj, 105, 55);
                testPoint(obj, 129, 69);
                testPoint(obj, 131, 55);
                testPoint(obj, 105, 71);
                testPoint(obj, -45, -65);
                testPoint(obj, 255, 175);
                testObject(obj, 95, 45);
                testObject(obj, 125, 65);
                testObject(obj, 131, 55);
                testObject(obj, -48, -68);
                testObject(obj, 250, 170);
            }
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame != 2) {
                return;
            }
            trace("// after render");
            testAll();
            trace("// done");
        }
    }
}
