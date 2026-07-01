package {
    import flash.display.*;
    import flash.geom.Rectangle;

    [SWF(width="440", height="680", backgroundColor="0xDDDDDD", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            addRow(20,  false, false);
            addRow(240, true,  false);
            addRow(460, true,  true);
        }

        private function addRow(yPos:Number, childHasGrid:Boolean, childIsShape:Boolean):void {
            var parent:Sprite = new Sprite();
            drawPanel(parent.graphics);
            parent.scale9Grid = new Rectangle(20, 20, 160, 60);

            var child:DisplayObject;
            if (childIsShape) {
                var shp:Shape = new Shape();
                drawChild(shp.graphics);
                if (childHasGrid) shp.scale9Grid = new Rectangle(8, 8, 14, 14);
                child = shp;
            } else {
                var spr:Sprite = new Sprite();
                drawChild(spr.graphics);
                if (childHasGrid) spr.scale9Grid = new Rectangle(8, 8, 14, 14);
                child = spr;
            }
            child.x = 70;
            child.y = 20;
            child.width = 60;
            child.height = 60;
            parent.addChild(child);

            parent.x = 20;
            parent.y = yPos;
            parent.width = 400;
            parent.height = 200;
            addChild(parent);
        }

        private function drawPanel(g:Graphics):void {
            g.beginFill(0x14242E);
            g.drawRoundRect(0, 0, 200, 100, 20, 20);
            g.endFill();
            g.beginFill(0x2A4D5E);
            g.drawRoundRect(2, 2, 196, 96, 18, 18);
            g.endFill();
            g.beginFill(0xC4B58E);
            g.drawRect(3, 3, 6, 6);
            g.drawRect(191, 3, 6, 6);
            g.drawRect(3, 91, 6, 6);
            g.drawRect(191, 91, 6, 6);
            g.endFill();
            g.beginFill(0x796646);
            g.drawRect(3, 48, 4, 4);
            g.drawRect(193, 48, 4, 4);
            g.drawRect(98, 3, 4, 4);
            g.drawRect(98, 93, 4, 4);
            g.endFill();
        }

        private function drawChild(g:Graphics):void {
            g.beginFill(0x882222);
            g.drawRoundRect(0, 0, 30, 30, 10, 10);
            g.endFill();
            g.beginFill(0xCC4444);
            g.drawRoundRect(2, 2, 26, 26, 8, 8);
            g.endFill();
            g.beginFill(0xFFEE99);
            g.drawRect(3, 3, 4, 4);
            g.drawRect(23, 3, 4, 4);
            g.drawRect(3, 23, 4, 4);
            g.drawRect(23, 23, 4, 4);
            g.endFill();
        }
    }
}
