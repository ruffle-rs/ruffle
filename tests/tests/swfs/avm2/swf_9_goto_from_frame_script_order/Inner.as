package {
    import flash.display.MovieClip;

    public class Inner extends MovieClip {
        public function Inner() {
            trace("Inner constructor (parent " + (parent ? parent.name : null) + ")");
            if (parent && parent.name == "dkD") {
                var test:Test = Test(parent.parent);
                trace("  dkC.gotoAndStop(3) from Inner of dkD");
                test.dkC.gotoAndStop(3);
                trace("  after: dkC.inner = " + test.dkC.inner);
            }
        }

        override public function toString():String {
            return "[Inner of " + (parent ? parent.name : null) + "]";
        }
    }
}
