package {
    import flash.display.Shader;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.utils.ByteArray;

    [SWF(width="120", height="60", frameRate="1", backgroundColor="#336699")]
    public class Test extends Sprite {
        [Embed(source="White.pbj", mimeType="application/octet-stream")]
        private static const WhiteBytes:Class;

        public function Test() {
            addMaskedShape(this);

            var container:Sprite = new Sprite();
            container.x = 60;
            addChild(container);
            addMaskedShape(container);

            // The second shader also has an ancestor mask.
            var outerMask:Shape = rectangle(60, 0, 60, 30, 0xFFFFFF);
            addChild(outerMask);
            container.mask = outerMask;
        }

        private function addMaskedShape(parent:Sprite):void {
            var foreground:Shape = rectangle(10, 10, 30, 30, 0x000000);
            var stencil:Shape = rectangle(25, 5, 25, 45, 0xFFFFFF);
            parent.addChild(foreground);
            parent.addChild(stencil);
            foreground.mask = stencil;
            foreground.blendShader = new Shader(new WhiteBytes() as ByteArray);
        }

        private function rectangle(x:int, y:int, width:int, height:int, color:uint):Shape {
            var shape:Shape = new Shape();
            shape.graphics.beginFill(color);
            shape.graphics.drawRect(x, y, width, height);
            shape.graphics.endFill();
            return shape;
        }
    }
}
