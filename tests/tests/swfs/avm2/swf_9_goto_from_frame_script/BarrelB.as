package {
    import flash.display.MovieClip;

    public dynamic class BarrelB extends MovieClip {
        public var inner:MovieClip;

        public function BarrelB() {
            addFrameScript(0, frame1);
        }

        public function frame1():void {
            trace(name + " frame1, inner = " + inner);
            stop();
        }
    }
}
