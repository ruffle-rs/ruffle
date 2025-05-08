package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        public function Test() {
            var shape = new Shape();
            shape.name = "shape";
            shape.graphics.beginFill(0xFF0000);
            shape.graphics.drawRect(0, 0, 100, 100);
            shape.graphics.endFill();

            addChild(shape);

            trace("// stage root - onscreen");
            Print(shape);

            trace("// stage root - offscreen");
            var savedStage = this.stage;
            savedStage.removeChild(this);
            Print(shape);
            savedStage.addChild(this);

            var loader = new Loader();
            loader.name = "loader";
            loader.load(new URLRequest("EmptyContainer.swf"));

            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e: Event) : void {
                trace("// load complete");

                var container: MovieClip = e.target.content;
                //container.name = "container"; // makes fp angry.. okay
                container.addChild(shape);

                trace("// loader root - offscreen");
                Print(shape);

                addChild(container);

                trace("// loader root - onscreen");
                Print(shape);
            });
        }

        public function Print(shape: Shape) : void
        {
            trace(shape.hitTestPoint(50, 50));
            shape.root.x += 100;
            trace(shape.hitTestPoint(50, 50));
            shape.root.x -= 100;
        }
    }
}
