package {
    import flash.display.MovieClip;

    public dynamic class Jumper extends MovieClip {
        public function Jumper() {
            addFrameScript(0, frame1);
        }

        public function frame1():void {
            trace("jumper frame1: gotoAndStop(2)");
            gotoAndStop(2);
            trace("jumper frame1 end");
        }
    }
}
