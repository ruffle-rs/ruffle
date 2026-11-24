package {
    import flash.display.MovieClip;

    public class Starter extends MovieClip {
        public function Starter() {
            var test:Test = Test(parent.parent);

            trace("Starter constructor: target.gotoAndStop(3)");
            test.target.gotoAndStop(3);
            trace("Starter constructor end, target.inner = " + test.target.inner);
        }
    }
}
