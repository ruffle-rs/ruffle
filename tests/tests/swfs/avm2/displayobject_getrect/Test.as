package {
    import flash.display.DisplayObject;
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            // First child: Ordinary `Shape`
            var firstChild:DisplayObject = this.getChildAt(0);
            // Second child: Ordinary `MorphShape`
            var secondChild:DisplayObject = this.getChildAt(1);
            // Third child: Modified `Shape`; `edgeBounds` and `shapeBounds` do not match where the shape actually is
            var thirdChild:DisplayObject = this.getChildAt(2);

            trace(firstChild);
            trace(secondChild);

            trace(firstChild.getBounds(null));
            trace(firstChild.getRect(null));
            trace(secondChild.getBounds(null));
            trace(secondChild.getRect(null));
            trace(thirdChild.getBounds(null));
            trace(thirdChild.getRect(null));

            trace(this.getBounds(null));
            trace(this.getRect(null));
            
            var newMC:MovieClip = new MovieClip();
            newMC.graphics.beginFill(0xFF0000);
            newMC.graphics.lineStyle(3, 0x00FF00);
            newMC.graphics.drawCircle(20, 20, 8);
            
            newMC.x = 120;
            newMC.y = 120;

            trace(newMC.getBounds(null));
            trace(newMC.getRect(null));

            addChild(newMC);

            trace(newMC.getBounds(null));
            trace(newMC.getRect(null));
            trace(this.getBounds(null));
            trace(this.getRect(null));
        }
    }
}
