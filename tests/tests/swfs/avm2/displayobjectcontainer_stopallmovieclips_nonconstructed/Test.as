package {
    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.display.MovieClip;

    public class Test extends MovieClip {

        public function Test() {
            var self:Test = this;
            addEventListener("enterFrame", function():void {
                trace("Own frame: " + self.currentFrame);
                trace("Entering frame, self.numChildren: " + self.numChildren);
                for(var i:int = 0; i < self.numChildren; i ++;) {
                    var child:DisplayObject = self.getChildAt(i);

                    trace("    " + child);

                    if (child === null) {
                        trace("Stopping all MCs");
                        self.stopAllMovieClips();
                    }

                    if (child is MovieClip) {
                        var childMC:MovieClip = child;
                        trace("        frame: " + childMC.currentFrame);
                    }
                }
                trace("");
            });

            addEventListener("frameConstructed", function():void {
                trace("Own frame: " + self.currentFrame);
                trace("Constructing frame, self.numChildren: " + self.numChildren);
                for(var i:int = 0; i < self.numChildren; i ++;) {
                    var child:DisplayObject = self.getChildAt(i);

                    trace("    " + child);

                    if (child is MovieClip) {
                        var childMC:MovieClip = child;
                        trace("        frame: " + childMC.currentFrame);
                    }
                }
                trace("");
            });
        }
    }
}
