package  {
    import flash.display.*;

    [SWF(width="400", height="400")]
    public class Test extends MovieClip {
        public function Test() {
            var rect = new Sprite();
            rect.graphics.lineStyle(2, 0xFF0000);
            rect.graphics.drawRect(0, 0, 400, 400);
            rect.graphics.drawRect(199, 199, 2, 2);
            addChild(rect);
            trace("Loaded!");
        }
    }
}
