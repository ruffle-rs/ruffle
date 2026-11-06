package {
    import flash.display.Shader;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.utils.ByteArray;

    [SWF(width="60", height="60", frameRate="1", backgroundColor="#336699")]
    public class Test extends Sprite {
        [Embed(source="White.pbj", mimeType="application/octet-stream")]
        private static const WhiteBytes:Class;

        private var foreground:Shape = new Shape();

        public function Test() {
            foreground.graphics.beginFill(0x336699);
            foreground.graphics.drawRect(15, 15, 30, 10);
            foreground.graphics.drawRect(15, 35, 30, 10);
            foreground.graphics.drawRect(15, 25, 10, 10);
            foreground.graphics.drawRect(35, 25, 10, 10);
            foreground.graphics.endFill();
            addChild(foreground);

            // A constant output makes every pixel written by the blend visible.
            foreground.blendShader = new Shader(new WhiteBytes() as ByteArray);
        }
    }
}
