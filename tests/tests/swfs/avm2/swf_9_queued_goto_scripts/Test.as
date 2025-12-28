package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {

        public function Test() {
            super();
            this.addFrameScript(0, this.frame1, 1, this.frame2, 2, this.frame3, 3, this.frame4);
            this.addEventListener("enterFrame", function(e) {
                trace("enterFrame called");
            });
            this.addEventListener("frameConstructed", function(e) {
                trace("frameConstructed called");
            });
            this.addEventListener("exitFrame", function(e) {
                trace("exitFrame called");
            });
        }

        public function frame1():void {
            trace("frame1");
            nextFrame();
        }

        public function frame2():void {
            trace("frame2");
            gotoAndPlay(4);
        }

        public function frame3():void {
            trace("frame3");
            gotoAndStop(3);
        }

        public function frame4():void {
            trace("frame4");
            prevFrame();
        }
    }
}

