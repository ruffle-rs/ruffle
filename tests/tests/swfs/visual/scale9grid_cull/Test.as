package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // A child sliced by its container's grid can land far from its plain bounds, so it must
    // not be culled by them. The child sits in the middle band of a container stretched
    // tenfold: plainly scaled it would cover x = 205..395, entirely off this 100px stage,
    // while sliced it starts at x = 34.
    [SWF(width="100", height="80", backgroundColor="#101010")]
    public class Test extends MovieClip {
        public function Test() {
            var container:MovieClip = new MovieClip();
            container.graphics.beginFill(0x303030);
            container.graphics.drawRect(0, 0, 60, 60);
            container.graphics.endFill();

            var child:Shape = new Shape();
            child.graphics.beginFill(0x40D040);
            child.graphics.drawRect(20.5, 10, 19, 40);
            child.graphics.endFill();
            container.addChild(child);

            container.scale9Grid = new Rectangle(20, 5, 20, 50);
            container.scaleX = 10;
            container.y = 10;
            addChild(container);
        }
    }
}
