package {
    import flash.display.MovieClip;

    public dynamic class Barrel extends MovieClip {
        public var inner:MovieClip;

        public function Barrel() {
            addFrameScript(0, frame1, 2, frame3);
        }

        public function frame1():void {
            stop();
        }

        public function frame3():void {
            // The frame script of "off" runs once or twice, depending on the
            // Flash Player run, so it isn't traced.
            if (name != "off") {
                trace(name + " frame3, inner = " + inner);
            }

            stop();
        }
    }
}
