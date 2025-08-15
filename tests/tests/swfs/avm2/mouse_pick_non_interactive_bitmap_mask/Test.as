package {
    import flash.display.MovieClip;
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.display.Shape;
    import flash.events.MouseEvent;

    [SWF(width="50", height="50")]
    public class Test extends MovieClip {
        public function Test() {
            stage.scaleMode = "noScale";

            var parent: MovieClip = new MovieClip();

            var target: Bitmap = new Bitmap(new BitmapData(25, 50, false, 0xFF00FF));

            var mask: Shape = new Shape();
            mask.graphics.beginFill(0xFF0000);
            mask.graphics.drawRect(0, 0, 50, 25);
            mask.graphics.endFill();

            target.mask = mask;
            parent.addChild(target);
            parent.addChild(mask);
            addChild(parent);

            target.name = "Target";
            mask.name = "Mask";
            parent.name = "Parent";

            stage.addEventListener("mouseDown", function(e: MouseEvent):void {
                trace("mouseDown: target=" + e.target + " " + e.target.name + " stageX=" + e.stageX + " stageY=" + e.stageY);
            });
        }
    }
}
