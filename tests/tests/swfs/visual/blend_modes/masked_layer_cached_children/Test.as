package {
    import flash.display.BlendMode;
    import flash.display.Sprite;

    [SWF(width="160", height="80", backgroundColor="#DCE8F5", frameRate="12")]
    public class Test extends Sprite {
        public function Test() {
            var content:Sprite = new Sprite();
            addChild(content);

            // #18734 exposed cached character artwork which should have been
            // hidden by the later title-screen surface.
            content.addChild(makeModel(0xC0392B, 20));
            content.addChild(makeModel(0x7D3C98, 70));

            // Reduce the masked Layer path from the affected title screen.
            // This later surface must continue to cover the model artwork.
            var cover:Sprite = new Sprite();
            cover.graphics.beginFill(0xDCE8F5);
            cover.graphics.drawRect(0, 0, 160, 80);
            cover.graphics.endFill();
            cover.blendMode = BlendMode.LAYER;
            content.addChild(cover);

            var stageMask:Sprite = new Sprite();
            stageMask.graphics.beginFill(0xFFFFFF);
            stageMask.graphics.drawRect(0, 0, 160, 80);
            stageMask.graphics.endFill();
            addChild(stageMask);
            content.mask = stageMask;
        }

        private function makeModel(color:uint, x:Number):Sprite {
            var model:Sprite = new Sprite();
            model.graphics.beginFill(color);
            model.graphics.drawRect(0, 0, 36, 40);
            model.graphics.endFill();
            model.x = x;
            model.y = 20;
            model.cacheAsBitmap = true;
            return model;
        }
    }
}
