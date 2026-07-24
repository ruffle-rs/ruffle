package {
    import flash.display.*;
    import flash.geom.Rectangle;

    [SWF(width="280", height="100", backgroundColor="0xDDDDDD", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            var btn:Sprite = new Sprite();
            var g:Graphics = btn.graphics;
            g.beginFill(0x113366);
            g.drawRoundRect(0, 0, 60, 40, 16, 16);
            g.endFill();
            g.beginFill(0x6699CC);
            g.drawRoundRect(2, 2, 56, 36, 12, 12);
            g.endFill();
            btn.scale9Grid = new Rectangle(8, 8, 44, 24);
            btn.width = 240;
            btn.height = 60;
            btn.x = 20;
            btn.y = 20;
            addChild(btn);
        }
    }
}
