package {
    import flash.display.*;
    import flash.geom.Rectangle;

    [SWF(width="400", height="240", backgroundColor="0xDDDDDD", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            var panel:Sprite = new Sprite();
            var g:Graphics = panel.graphics;

            g.beginFill(0x14242E);
            g.drawRoundRect(0, 0, 120, 80, 20, 20);
            g.endFill();

            g.beginFill(0x2A4D5E);
            g.drawRoundRect(2, 2, 116, 76, 18, 18);
            g.endFill();

            g.beginFill(0xC4B58E);
            g.drawRect(3, 3, 6, 6);
            g.drawRect(111, 3, 6, 6);
            g.drawRect(3, 71, 6, 6);
            g.drawRect(111, 71, 6, 6);
            g.endFill();

            g.beginFill(0x796646);
            g.drawRect(3, 38, 4, 4);    // left edge (native x, stretches y)
            g.drawRect(113, 38, 4, 4);  // right edge (native x, stretches y)
            g.drawRect(58, 3, 4, 4);    // top edge (stretches x, native y)
            g.drawRect(58, 73, 4, 4);   // bottom edge (stretches x, native y)
            g.endFill();

            panel.scale9Grid = new Rectangle(12, 12, 96, 56);
            panel.width = 360;
            panel.height = 200;
            panel.x = 20;
            panel.y = 20;
            addChild(panel);
        }
    }
}
