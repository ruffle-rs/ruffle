package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public var nextGotoFrame:int;

        public function Test() {
            super();

            play();

            var self:Test = this;
            addEventListener("enterFrame",function():* {
                trace("enterFrame : currentFrame=" + self.currentFrame);
            });

            addFrameScript(0,frame1_1,1,frame2_1,2,frame3_1,3,frame4_1,4,frame5_1,5,frame6_1);
        }

        public function frame1_1() : void {
            trace("Entered frame 1");
            trace("// gotoAndStop(0)");
            try {
                gotoAndStop(0);
            }
            catch(e:Error) {
                trace("// Error! " + e.getStackTrace());
            }
        }

        public function frame2_1() : void {
            trace("Entered frame 2");
            trace("// gotoAndStop(\'0\')");
            try {
                gotoAndStop("0");
            }
            catch(e:Error) {
                trace("// Error! " + e.getStackTrace());
            }
        }

        public function frame3_1() : void {
            trace("Entered frame 3");
            trace("// gotoAndStop(undefined)");
            try {
                gotoAndStop(undefined);
            }
            catch(e:Error) {
                trace("// Error! " + e.getStackTrace());
            }
        }

        public function frame4_1() : void {
            trace("Entered frame 4");
            trace("// gotoAndPlay({})");
            try {
                gotoAndPlay({});
            }
            catch(e:Error) {
                trace("// Error! " + e.getStackTrace());
            }
        }

        public function frame5_1() : void {
            trace("Entered frame 5");
            trace("// gotoAndPlay(-Infinity)");
            try {
                gotoAndPlay(-Infinity);
            }
            catch(e:Error) {
                trace("// Error! " + e.getStackTrace());
            }
        }

        public function frame6_1() : void {
            trace("Entered frame 6");
            trace("// stop()");
            stop();
            trace("Done testing cases that don\'t goto! Now let\'s test some cases that DO goto.");
            trace("// addFrameScript(new frame scripts)");
            addFrameScript(0,frame1_2,1,frame2_2,2,frame3_2,3,frame4_2,4,null,5,frame6_2);
            trace("// gotoAndPlay(1)");
            this.nextGotoFrame = 2;
            gotoAndPlay(1);
        }

        public function frame1_2() : void {
            trace("Entered frame 1");
            trace("// gotoAndPlay(" + this.nextGotoFrame + ")");
            gotoAndPlay(this.nextGotoFrame);
            this.nextGotoFrame++;
        }

        public function frame2_2() : void {
            trace("Entered frame 2");
            trace("// gotoAndPlay(-1)");
            try {
                gotoAndPlay(-1);
            }
            catch(e:Error) {
                trace("// Error! " + e.getStackTrace());
            }
        }

        public function frame3_2() : void {
            trace("Entered frame 3");
            trace("// gotoAndPlay(999)");
            try {
                gotoAndPlay(999);
            }
            catch(e:Error) {
                trace("// Error! " + e.getStackTrace());
            }
        }

        public function frame4_2() : void {
            trace("Entered frame 4");
            trace("// stop()");
            stop();
        }

        public function frame6_2() : void {
            trace("Entered frame 6");
            trace("// stop()");
            stop();
        }
    }
}

