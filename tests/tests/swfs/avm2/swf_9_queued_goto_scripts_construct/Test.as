package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public var counter:int = 0;

        public var theSprite:TheSprite;

        public function Test() {
            super();
            var self:Test = this;
            addEventListener("enterFrame",function(e:*):void {
                trace("enterFrame dispatched");
                trace("Number of root children: " + self.numChildren);
                trace("placed theSprite: " + self.theSprite);
                trace("theSprite: " + theSprite);
            });
            addEventListener("frameConstructed",function(e:*):void {
                trace("frameConstructed dispatched");
                trace("Number of root children: " + self.numChildren);
                trace("placed theSprite: " + self.theSprite);
                trace("theSprite: " + theSprite);
            });
            addEventListener("exitFrame",function(e:*):void {
                trace("exitFrame dispatched");
                trace("Number of root children: " + self.numChildren);
                trace("placed theSprite: " + self.theSprite);
                trace("theSprite: " + theSprite);
            });
            addFrameScript(1,frame2,2,frame3);
            gotoAndStop(2);
            trace("Test constructor complete");
        }

        public function frame2() : void {
            trace("frame 2 framescript: On frame 2");
            gotoAndStop(3);
            trace("frame 2 framescript: Ran gotoAndStop(3) from frame 2");
        }

        public function frame3() : void {
            trace("frame 3 framescript: On frame 3");
            trace("frame 3 framescript: placed theSprite is " + this.theSprite);
            if(counter < 2) {
                counter++;
                gotoAndStop(3);
                trace("frame 3 framescript: Ran gotoAndStop(3) from frame 3");
            }
        }
    }
}
