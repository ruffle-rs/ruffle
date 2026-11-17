package {
    import flash.display.Shader;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.utils.ByteArray;

    [SWF(width="60", height="60", frameRate="1", backgroundColor="#336699")]
    public class Test extends Sprite {
        [Embed(source="Tint.pbj", mimeType="application/octet-stream")]
        private static const TintBytes:Class;

        private var foreground:Shape = new Shape();
        private var shader:Shader;
        private var frames:int = 0;

        public function Test() {
            foreground.graphics.beginFill(0x336699);
            foreground.graphics.drawRect(0, 0, 60, 60);
            foreground.graphics.endFill();
            addChild(foreground);

            shader = new Shader(new TintBytes() as ByteArray);
            shader.data.tint.value = [1, 1, 1, 1];
            foreground.blendShader = shader;

            shader.data.tint.value = [1, 0, 0, 1];
            addEventListener(Event.ENTER_FRAME, reassignShader);
        }

        private function reassignShader(event:Event):void {
            if (++frames == 2) {
                foreground.blendShader = shader;
                removeEventListener(Event.ENTER_FRAME, reassignShader);
            }
        }
    }
}
