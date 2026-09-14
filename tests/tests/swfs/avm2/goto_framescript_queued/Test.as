package test_fla {
    import flash.display.MovieClip;

    public dynamic class MainTimeline extends MovieClip {
        internal var allFinished:Boolean = false;

        public function MainTimeline() {
            super();
            addFrameScript(0, this.frame1, 1, this.frame2, 2, this.frame3, 3, this.frame4, 4, this.frame5, 5, this.frame6, 6, this.frame7, 7, this.frame8, 8, this.frame9, 9, this.frame10, 10, this.frame11, 11, this.frame12);

            addEventListener("frameConstructed", function(e) {
                if(!allFinished) {
                    trace("! frameConstructed");
                }
            });

            addEventListener("exitFrame", function(e) {
                if(!allFinished) {
                    trace("! exitFrame");
                }
            });
        }

        public function traceInfo() {
            try {
                trace("frame: " + currentFrame + ", isPlaying: " + isPlaying);
            } catch(e:Error) {
                // Below SWFv13, `isPlaying` is inaccessible.
                trace("frame: " + currentFrame);
            }
        }

        internal function frame1():* {
            this.traceInfo();
            gotoAndPlay(2);
            this.traceInfo();
            stop();
            this.traceInfo();
        }

        internal function frame2():* {
            this.traceInfo();
        }

        internal function frame3():* {
            this.traceInfo();
            gotoAndStop(4);
            this.traceInfo();
        }

        internal function frame4():* {
            this.traceInfo();
            play();
            this.traceInfo();
        }

        internal function frame5():* {
            this.traceInfo();
            play();
            this.traceInfo();
            stop();
            this.traceInfo();
            gotoAndPlay(6);
            this.traceInfo();
        }

        internal function frame6():* {
            this.traceInfo();
            gotoAndStop(7);
            this.traceInfo();
            play();
            this.traceInfo();
            stop();
            this.traceInfo();
        }

        internal function frame7():* {
            this.traceInfo();
            gotoAndStop(8);
            this.traceInfo();
        }

        internal function frame8():* {
            this.traceInfo();
            gotoAndPlay(9);
            this.traceInfo();
        }

        internal function frame9():* {
            this.traceInfo();
        }

        internal function frame10():* {
            this.traceInfo();
            play();
            this.traceInfo();
            gotoAndStop(11);
            this.traceInfo();
        }

        internal function frame11():* {
            this.traceInfo();
            allFinished = true;
        }

        internal function frame12():* {
            this.traceInfo();
            trace("We should not be here!");
            stop();
        }

        // Overrides

        override public function play():void {
            trace("// play()");
            super.play();
        }

        override public function stop():void {
            trace("// stop()");
            super.stop();
        }

        override public function gotoAndStop(frame:Object, scene:String = null):void {
            trace("// gotoAndStop(" + frame + ")");
            super.gotoAndStop(frame);
        }

        override public function gotoAndPlay(frame:Object, scene:String = null):void {
            trace("// gotoAndPlay(" + frame + ")");
            super.gotoAndPlay(frame);
        }
    }
}
